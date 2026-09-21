use std::net::IpAddr;
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

/// Rechecks DNS immediately before external access. Every answer must be public.
pub async fn validate_media_url_network(raw_url: &str) -> Result<String, UrlValidationError> {
    let parsed = parsed_public_url(raw_url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| UrlValidationError::MalformedUrl("missing host".to_string()))?;

    if host.parse::<IpAddr>().is_err() {
        let port = parsed.port_or_known_default().ok_or_else(|| {
            UrlValidationError::MalformedUrl("URL has no resolvable port".to_string())
        })?;
        let addresses = tokio::net::lookup_host((host, port))
            .await
            .map_err(|error| UrlValidationError::HostResolutionFailed(error.to_string()))?;
        let mut found = false;
        for address in addresses {
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
    }

    Ok(raw_url.trim().to_string())
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
            "http://169.254.10.20/video",
            "http://[::1]/video",
            "http://[fc00::1]/video",
            "http://[fe80::1]/video",
            "http://[::ffff:127.0.0.1]/video",
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
}
