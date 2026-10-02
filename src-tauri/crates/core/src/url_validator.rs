use std::net::{IpAddr, SocketAddr};
use thiserror::Error;
use url::{Host, Url};

#[derive(Error, Debug, PartialEq)]
pub enum UrlValidationError {
    #[error("URL cannot be empty")]
    EmptyUrl,
    #[error("Only HTTP and HTTPS URLs are supported: received '{0}'")]
    UnsupportedScheme(String),
    #[error("Invalid characters or control bytes detected in URL")]
    InvalidCharacters,
    #[error("URL format is malformed: {0}")]
    MalformedUrl(String),
    #[error("URL exceeds maximum permitted length (2048 characters)")]
    TooLong,
    #[error("Localhost and private network addresses are prohibited for security")]
    ProhibitedHost,
    #[error("Could not resolve URL host: {0}")]
    HostResolutionFailed(String),
}

fn is_prohibited_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.is_multicast()
                || ip.octets()[0] == 0
                || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
                || (ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0)
                || (ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 2)
                || (ip.octets()[0] == 198 && (18..=19).contains(&ip.octets()[1]))
                || (ip.octets()[0] == 198 && ip.octets()[1] == 51 && ip.octets()[2] == 100)
                || (ip.octets()[0] == 203 && ip.octets()[1] == 0 && ip.octets()[2] == 113)
                || ip.octets()[0] >= 240
        }
        IpAddr::V6(ip) => {
            if let Some(ipv4) = ip.to_ipv4_mapped() {
                return is_prohibited_ip(IpAddr::V4(ipv4));
            }
            ip.is_loopback()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.segments()[0] & 0xffc0 == 0xfe80
                || ip.segments()[0] & 0xfe00 == 0xfc00
                || (ip.segments()[0] == 0x2001 && ip.segments()[1] == 0x0db8)
        }
    }
}

fn parsed_public_url(raw_url: &str) -> Result<Url, UrlValidationError> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err(UrlValidationError::EmptyUrl);
    }
    if trimmed.len() > 2048 {
        return Err(UrlValidationError::TooLong);
    }
    if trimmed.chars().any(char::is_control) {
        return Err(UrlValidationError::InvalidCharacters);
    }

    let parsed =
        Url::parse(trimmed).map_err(|error| UrlValidationError::MalformedUrl(error.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(UrlValidationError::UnsupportedScheme(
            parsed.scheme().to_string(),
        ));
    }

    let host = parsed
        .host()
        .ok_or_else(|| UrlValidationError::MalformedUrl("missing host".to_string()))?;
    match host {
        Host::Domain(domain) => {
            let normalized = domain.trim_end_matches('.').to_ascii_lowercase();
            if normalized == "localhost" || normalized.ends_with(".localhost") {
                return Err(UrlValidationError::ProhibitedHost);
            }
        }
        Host::Ipv4(ip) => {
            if is_prohibited_ip(IpAddr::V4(ip)) {
                return Err(UrlValidationError::ProhibitedHost);
            }
        }
        Host::Ipv6(ip) => {
            if is_prohibited_ip(IpAddr::V6(ip)) {
                return Err(UrlValidationError::ProhibitedHost);
            }
        }
    }

    Ok(parsed)
}

pub fn validate_media_url(raw_url: &str) -> Result<String, UrlValidationError> {
    parsed_public_url(raw_url).map(|_| raw_url.trim().to_string())
}

/// Resolve and validate a public URL. All returned addresses are pinned into
/// the in-process HTTP client to avoid a second DNS lookup at connection time.
async fn resolve_public_url(raw_url: &str) -> Result<(Url, Vec<SocketAddr>), UrlValidationError> {
    let parsed = parsed_public_url(raw_url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| UrlValidationError::MalformedUrl("missing host".to_string()))?;

    let port = parsed.port_or_known_default().ok_or_else(|| {
        UrlValidationError::MalformedUrl("URL has no resolvable port".to_string())
    })?;
    let addresses: Vec<SocketAddr> = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|error| UrlValidationError::HostResolutionFailed(error.to_string()))?
            .collect();
        addresses
    };
    let mut found = false;
    for address in &addresses {
        found = true;
        if is_prohibited_ip(address.ip()) {
            return Err(UrlValidationError::ProhibitedHost);
        }
    }
    if !found {
        return Err(UrlValidationError::HostResolutionFailed(
            "host returned no addresses".to_string(),
        ));
    }
    Ok((parsed, addresses))
}

/// Rechecks DNS immediately before external access. Every answer must be public.
pub async fn validate_media_url_network(raw_url: &str) -> Result<String, UrlValidationError> {
    resolve_public_url(raw_url)
        .await
        .map(|(url, _)| url.to_string())
}

fn redirect_target(base: &Url, location: &str) -> Result<Url, String> {
    let target = base.join(location).map_err(|e| e.to_string())?;
    parsed_public_url(target.as_str()).map_err(|e| e.to_string())
}

/// GET an untrusted URL while validating and pinning every redirect destination.
pub async fn get_public_url(
    raw_url: &str,
    headers: &[(&str, &str)],
) -> Result<reqwest::Response, String> {
    let mut url = parsed_public_url(raw_url).map_err(|e| e.to_string())?;
    for _ in 0..=10 {
        let (validated, addresses) = resolve_public_url(url.as_str())
            .await
            .map_err(|e| e.to_string())?;
        url = validated;
        let host = url.host_str().ok_or("URL has no host")?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve_to_addrs(host, &addresses)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;
        let mut request = client.get(url.clone());
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await.map_err(|e| e.to_string())?;
        if !response.status().is_redirection() {
            return Ok(response);
        }
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .ok_or("Redirect response omitted Location")?
            .to_str()
            .map_err(|_| "Redirect Location is not valid text")?;
        url = redirect_target(&url, location)?;
    }
    Err("Too many redirects".to_string())
}

pub use validate_media_url as validate_url;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_media_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validate_media_url("http://vimeo.com/123456").is_ok());
        assert!(validate_media_url("https://soundcloud.com/artist/track").is_ok());
    }

    #[test]
    fn test_rejects_empty_or_whitespace() {
        assert_eq!(validate_media_url(""), Err(UrlValidationError::EmptyUrl));
        assert_eq!(
            validate_media_url("   \t  "),
            Err(UrlValidationError::EmptyUrl)
        );
    }

    #[test]
    fn test_rejects_unsupported_schemes() {
        assert_eq!(
            validate_media_url("file:///etc/passwd"),
            Err(UrlValidationError::UnsupportedScheme("file".to_string()))
        );
        assert_eq!(
            validate_media_url("ftp://server.com/file.mp4"),
            Err(UrlValidationError::UnsupportedScheme("ftp".to_string()))
        );
    }

    #[test]
    fn test_rejects_control_characters() {
        assert_eq!(
            validate_media_url("https://youtube.com/watch\0bad"),
            Err(UrlValidationError::InvalidCharacters)
        );
        assert_eq!(
            validate_media_url("https://youtube.com/watch\nbad"),
            Err(UrlValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_rejects_private_ipv4_and_ipv6_literals() {
        for url in [
            "http://127.0.0.1/video",
            "http://10.0.0.1/video",
            "http://172.16.1.2/video",
            "http://192.168.1.2/video",
            "http://100.64.0.1/video",
            "http://192.0.0.1/video",
            "http://198.18.0.1/video",
            "http://240.0.0.1/video",
            "http://169.254.10.20/video",
            "http://[::1]/video",
            "http://[fc00::1]/video",
            "http://[fe80::1]/video",
            "http://[::ffff:127.0.0.1]/video",
            "http://[2001:db8::1]/video",
        ] {
            assert_eq!(
                validate_media_url(url),
                Err(UrlValidationError::ProhibitedHost),
                "{url} should be blocked"
            );
        }
    }

    #[test]
    fn test_rejects_localhost_names() {
        assert_eq!(
            validate_media_url("http://localhost/video"),
            Err(UrlValidationError::ProhibitedHost)
        );
        assert_eq!(
            validate_media_url("http://service.localhost/video"),
            Err(UrlValidationError::ProhibitedHost)
        );
    }

    #[test]
    fn redirect_targets_reject_private_destinations() {
        let base = Url::parse("https://public.example/path").unwrap();
        for location in [
            "http://127.0.0.1/admin",
            "//192.168.1.2/",
            "http://service.localhost/",
        ] {
            assert!(redirect_target(&base, location).is_err(), "{location}");
        }
        assert_eq!(
            redirect_target(&base, "../next").unwrap().as_str(),
            "https://public.example/next"
        );
    }
}
