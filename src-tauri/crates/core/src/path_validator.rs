use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PathValidationError {
    #[error("Directory path cannot be empty")]
    EmptyPath,
    #[error("Directory path contains invalid characters or null bytes")]
    InvalidCharacters,
    #[error("Directory path cannot be resolved or created: {0}")]
    DirectoryCreationFailed(String),
    #[error("Output path exists but is not a directory")]
    NotDirectory,
}

pub fn validate_and_ensure_directory(raw_dir: &str) -> Result<PathBuf, PathValidationError> {
    let trimmed = raw_dir.trim();
    if trimmed.is_empty() {
        return Err(PathValidationError::EmptyPath);
    }

    if trimmed.contains('\0') {
        return Err(PathValidationError::InvalidCharacters);
    }

    let path = PathBuf::from(trimmed);

    if path.exists() && !path.is_dir() {
        return Err(PathValidationError::NotDirectory);
    }

    // If directory does not exist, attempt to create it
    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            return Err(PathValidationError::DirectoryCreationFailed(e.to_string()));
        }
    }

    Ok(path)
}

pub fn sanitize_file_name(raw_name: &str, max_length: usize) -> String {
    // Replace reserved characters across Windows and Unix: \ / : * ? " < > |
    let sanitized: String = raw_name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            '\0'..='\x1f' => '_',
            other => other,
        })
        .collect();

    let trimmed = sanitized.trim();
    let effective_max = if max_length == 0 { 180 } else { max_length };

    if trimmed.chars().count() > effective_max {
        trimmed.chars().take(effective_max).collect()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        let dirty = "A / B \\ C : D * E ? F \" G < H > I | J.mp4";
        let cleaned = sanitize_file_name(dirty, 180);
        assert!(!cleaned.contains('/'));
        assert!(!cleaned.contains('\\'));
        assert!(!cleaned.contains(':'));
        assert!(!cleaned.contains('*'));
        assert!(!cleaned.contains('?'));
        assert!(!cleaned.contains('"'));
        assert!(!cleaned.contains('<'));
        assert!(!cleaned.contains('>'));
        assert!(!cleaned.contains('|'));
    }

    #[test]
    fn test_filename_truncation() {
        let long_title = "a".repeat(300);
        let cleaned = sanitize_file_name(&long_title, 100);
        assert_eq!(cleaned.len(), 100);
    }

    #[test]
    fn test_rejects_existing_file_as_output_directory() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("not-a-directory");
        std::fs::write(&file, b"data").unwrap();

        assert_eq!(
            validate_and_ensure_directory(&file.to_string_lossy()),
            Err(PathValidationError::NotDirectory)
        );
    }
}
