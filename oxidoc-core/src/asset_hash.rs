//! Asset hashing for cache-busting.
//!
//! Generates short content hashes and hashed filenames for CSS and JS assets.

use sha2::{Digest, Sha256};

/// Compute a short hash (first 8 chars of SHA-256) of the given content.
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result).chars().take(8).collect()
}

/// Generate a hashed filename from the original name and content.
///
/// Example: `oxidoc.css` with content "body { }" becomes `oxidoc.a1b2c3d4.css`
pub fn hashed_filename(original: &str, content: &[u8]) -> String {
    let hash = hash_content(content);

    if let Some(dot_pos) = original.rfind('.') {
        let (name, ext) = original.split_at(dot_pos);
        format!("{}.{}{}", name, hash, ext)
    } else {
        format!("{}.{}", original, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content_consistency() {
        let content = b"body { color: red; }";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 8);
    }

    #[test]
    fn test_hash_content_different() {
        let content1 = b"body { color: red; }";
        let content2 = b"body { color: blue; }";
        let hash1 = hash_content(content1);
        let hash2 = hash_content(content2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hashed_filename_with_extension() {
        let original = "oxidoc.css";
        let content = b"body { }";
        let hashed = hashed_filename(original, content);
        assert!(hashed.starts_with("oxidoc."));
        assert!(hashed.ends_with(".css"));
        assert_eq!(hashed.matches('.').count(), 2);
    }

    #[test]
    fn test_hashed_filename_without_extension() {
        let original = "loader";
        let content = b"console.log('hi')";
        let hashed = hashed_filename(original, content);
        assert!(hashed.starts_with("loader."));
        assert_eq!(hashed.matches('.').count(), 1);
    }

    #[test]
    fn test_hashed_filename_multiple_dots() {
        let original = "oxidoc.min.js";
        let content = b"var x=1";
        let hashed = hashed_filename(original, content);
        assert!(hashed.starts_with("oxidoc.min."));
        assert!(hashed.ends_with(".js"));
    }
}
