//! Subresource Integrity (SRI) hash generation for security.
//!
//! Generates SRI hashes (SHA-384 base64-encoded) for CSS and JS assets
//! to enable browser verification and prevent tampering.

use base64::Engine;
use sha2::{Digest, Sha384};

/// Generate an SRI hash string (SHA-384 base64) for the given content.
///
/// Returns a string in the format: `sha384-<base64-encoded-hash>`
///
/// # Arguments
/// * `content` - The raw bytes of the asset (CSS or JS)
///
/// # Example
/// ```
/// use oxidoc_core::sri::generate_sri_hash;
///
/// let css = b"body { color: red; }";
/// let sri = generate_sri_hash(css);
/// assert!(sri.starts_with("sha384-"));
/// assert!(sri.len() > 10);
/// ```
pub fn generate_sri_hash(content: &[u8]) -> String {
    let mut hasher = Sha384::new();
    hasher.update(content);
    let result = hasher.finalize();

    // Base64 encode the hash using the standard engine
    let b64_hash = base64::engine::general_purpose::STANDARD.encode(result);
    format!("sha384-{}", b64_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sri_hash_format() {
        let content = b"body { color: red; }";
        let sri = generate_sri_hash(content);
        assert!(sri.starts_with("sha384-"));
        // Base64 of 48-byte SHA-384 is approximately 64 characters
        assert!(sri.len() > 50);
    }

    #[test]
    fn sri_hash_consistency() {
        let content = b"const x = 1;";
        let sri1 = generate_sri_hash(content);
        let sri2 = generate_sri_hash(content);
        assert_eq!(sri1, sri2);
    }

    #[test]
    fn sri_hash_different_content() {
        let content1 = b"a { color: blue; }";
        let content2 = b"a { color: green; }";
        let sri1 = generate_sri_hash(content1);
        let sri2 = generate_sri_hash(content2);
        assert_ne!(sri1, sri2);
    }

    #[test]
    fn sri_hash_empty_content() {
        let empty = b"";
        let sri = generate_sri_hash(empty);
        assert!(sri.starts_with("sha384-"));
    }

    #[test]
    fn sri_hash_large_content() {
        let large: Vec<u8> = vec![b'x'; 100_000];
        let sri = generate_sri_hash(&large);
        assert!(sri.starts_with("sha384-"));
    }
}
