//! Incremental build cache based on SHA-256 hashes.
//!
//! Tracks file content hashes to detect changes between builds.
//! Cache is stored in `.oxidoc-cache.json` in the output directory.

use crate::asset_hash::hash_content;
use crate::error::{OxidocError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Persistent cache of file hashes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncrementalCache {
    /// Map of file path -> SHA-256 hash (8 chars)
    #[serde(default)]
    pub files: HashMap<String, String>,
}

impl IncrementalCache {
    const CACHE_FILENAME: &'static str = ".oxidoc-cache.json";

    /// Load cache from output directory, or return empty cache if it doesn't exist.
    pub fn load(output_dir: &Path) -> Result<Self> {
        let cache_path = output_dir.join(Self::CACHE_FILENAME);
        if !cache_path.exists() {
            return Ok(IncrementalCache::default());
        }

        let content = std::fs::read_to_string(&cache_path).map_err(|e| OxidocError::FileRead {
            path: cache_path.display().to_string(),
            source: e,
        })?;

        // Try to parse the cache; if it fails, just start with an empty cache
        match serde_json::from_str::<IncrementalCache>(&content) {
            Ok(cache) => Ok(cache),
            Err(_) => {
                tracing::warn!("Failed to parse cache file, starting fresh");
                Ok(IncrementalCache::default())
            }
        }
    }

    /// Save cache to output directory.
    pub fn save(&self, output_dir: &Path) -> Result<()> {
        let cache_path = output_dir.join(Self::CACHE_FILENAME);
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            tracing::warn!("Failed to serialize cache: {}", e);
            OxidocError::FileWrite {
                path: cache_path.display().to_string(),
                source: std::io::Error::other(e.to_string()),
            }
        })?;

        std::fs::write(&cache_path, json).map_err(|e| OxidocError::FileWrite {
            path: cache_path.display().to_string(),
            source: e,
        })
    }

    /// Check if a file needs rebuilding based on content hash.
    ///
    /// Returns `true` if the file is new or has changed.
    pub fn needs_rebuild(&self, path: &str, content: &[u8]) -> bool {
        let new_hash = hash_content(content);
        match self.files.get(path) {
            Some(cached_hash) => cached_hash != &new_hash,
            None => true, // New file
        }
    }

    /// Record a file's hash in the cache.
    pub fn record(&mut self, path: &str, content: &[u8]) {
        let hash = hash_content(content);
        self.files.insert(path.to_string(), hash);
    }

    /// Clear all cached hashes.
    pub fn clear(&mut self) {
        self.files.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_needs_rebuild_new_file() {
        let cache = IncrementalCache::default();
        assert!(cache.needs_rebuild("new.txt", b"content"));
    }

    #[test]
    fn test_cache_needs_rebuild_changed_file() {
        let mut cache = IncrementalCache::default();
        cache.record("file.txt", b"original");
        assert!(!cache.needs_rebuild("file.txt", b"original"));
        assert!(cache.needs_rebuild("file.txt", b"modified"));
    }

    #[test]
    fn test_cache_record_and_retrieve() {
        let mut cache = IncrementalCache::default();
        cache.record("test.txt", b"data");
        let hash = hash_content(b"data");
        assert_eq!(cache.files.get("test.txt"), Some(&hash));
    }

    #[test]
    fn test_cache_save_and_load() {
        let tmp = tempfile::TempDir::new().unwrap();
        let output = tmp.path();

        let mut cache = IncrementalCache::default();
        cache.record("file1.txt", b"content1");
        cache.record("file2.txt", b"content2");
        cache.save(output).unwrap();

        let loaded = IncrementalCache::load(output).unwrap();
        assert_eq!(loaded.files.len(), 2);
        assert_eq!(loaded.files.get("file1.txt"), cache.files.get("file1.txt"));
        assert_eq!(loaded.files.get("file2.txt"), cache.files.get("file2.txt"));
    }

    #[test]
    fn test_cache_load_missing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let loaded = IncrementalCache::load(tmp.path()).unwrap();
        assert_eq!(loaded.files.len(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = IncrementalCache::default();
        cache.record("file.txt", b"data");
        assert_eq!(cache.files.len(), 1);
        cache.clear();
        assert_eq!(cache.files.len(), 0);
    }
}
