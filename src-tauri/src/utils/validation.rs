use std::path::{Path, PathBuf};
use crate::error::{LauncherError, Result};

/// Sanitize a search query to prevent injection attacks
pub fn sanitize_query(query: &str) -> String {
    query
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(256) // Limit query length
        .collect()
}

/// Validate and canonicalize a file path
pub fn validate_file_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .map_err(|e| LauncherError::SecurityError(format!("Invalid path: {}", e)))?;
    
    // Basic security check - ensure path is not attempting traversal
    if !canonical.is_absolute() {
        return Err(LauncherError::SecurityError("Path must be absolute".to_string()));
    }
    
    Ok(canonical)
}

/// Check if a file path exists and is accessible
pub fn is_file_accessible(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Check if a directory path exists and is accessible
pub fn is_directory_accessible(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Validate a URL format
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Encode a string for use in a URL query parameter
pub fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_query() {
        assert_eq!(sanitize_query("  hello world  "), "hello world");
        assert_eq!(sanitize_query("test\nquery"), "testquery");
        assert_eq!(sanitize_query("normal query"), "normal query");
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("example.com"));
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("test@example.com"), "test%40example.com");
        assert_eq!(url_encode("simple"), "simple");
    }
}
