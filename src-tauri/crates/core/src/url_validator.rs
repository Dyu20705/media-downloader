use thiserror::Error;

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
}

pub fn validate_media_url(raw_url: &str) -> Result<String, UrlValidationError> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err(UrlValidationError::EmptyUrl);
    }

    if trimmed.len() > 2048 {
        return Err(UrlValidationError::TooLong);
    }

    // Reject null bytes, control characters, newlines
    if trimmed
        .chars()
        .any(|c| c.is_control() || c == '\0' || c == '\n' || c == '\r')
    {
        return Err(UrlValidationError::InvalidCharacters);
    }

    // Require HTTP or HTTPS scheme
    let lower = trimmed.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        let scheme = trimmed.split("://").next().unwrap_or("unknown").to_string();
        return Err(UrlValidationError::UnsupportedScheme(scheme));
    }

    // Basic domain check
    let after_scheme = if lower.starts_with("https://") {
        &trimmed[8..]
    } else {
        &trimmed[7..]
    };

    let domain_part = after_scheme.split('/').next().unwrap_or("");
    if domain_part.is_empty() || domain_part.contains(' ') {
        return Err(UrlValidationError::MalformedUrl(
            "Missing or invalid domain name".to_string(),
        ));
    }

    let host = domain_part.split(':').next().unwrap_or("").to_lowercase();
    if host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.starts_with("192.168.")
        || host.starts_with("10.")
        || (host.starts_with("172.") && {
            if let Some(second) = host.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()) {
                (16..=31).contains(&second)
            } else {
                false
            }
        })
    {
        return Err(UrlValidationError::ProhibitedHost);
    }

    Ok(trimmed.to_string())
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
        assert_eq!(
            validate_media_url("gopher://server.com"),
            Err(UrlValidationError::UnsupportedScheme("gopher".to_string()))
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
}
