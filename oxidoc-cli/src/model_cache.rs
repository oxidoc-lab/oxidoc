//! On-demand download and caching of the search embedding model.
//!
//! The GGUF embedding model is ~17 MB — too large to embed in a published
//! crate. Instead it is fetched from HuggingFace on first use and cached in
//! the user's XDG cache directory, keyed by SHA-256 so updates are transparent.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

const MODEL_URL: &str =
    "https://huggingface.co/fs90/bge-micro-v2-i1-GGUF/resolve/main/bge-micro-v2.i1-Q4_K_M.gguf";
const MODEL_SHA256: &str = "45242c638200a5696ba553446f146c242aef3ccaba09e4deaa0acfb37df5517d";
const MODEL_FILENAME: &str = "bge-micro-v2.gguf";

/// Override the cache lookup with a local model file (e.g. for CI or workspace dev).
const ENV_MODEL_PATH: &str = "OXIDOC_SEARCH_MODEL";

/// Load the search embedding model bytes, downloading on first use.
///
/// Resolution order:
///   1. `$OXIDOC_SEARCH_MODEL` if set and valid
///   2. Workspace asset at `oxidoc-cli/assets/models/bge-micro-v2.gguf` (dev mode)
///   3. Cached download at `$XDG_CACHE_HOME/oxidoc/models/<sha>/bge-micro-v2.gguf`
///   4. Fresh download from HuggingFace into the cache, then return it
pub fn load_search_model() -> Result<Vec<u8>, ModelError> {
    if let Some(path) = env_override() {
        return read_file(&path);
    }
    if let Some(path) = workspace_asset() {
        return read_file(&path);
    }

    let cache_path = cache_path()?;
    if cache_path.exists() {
        if let Ok(bytes) = read_file(&cache_path) {
            if verify_sha(&bytes) {
                return Ok(bytes);
            }
            tracing::warn!("Cached model failed checksum, re-downloading");
            let _ = fs::remove_file(&cache_path);
        }
    }

    download_to(&cache_path)?;
    let bytes = read_file(&cache_path)?;
    if !verify_sha(&bytes) {
        let _ = fs::remove_file(&cache_path);
        return Err(ModelError::ChecksumMismatch);
    }
    Ok(bytes)
}

fn env_override() -> Option<PathBuf> {
    let raw = std::env::var(ENV_MODEL_PATH).ok()?;
    let path = PathBuf::from(raw);
    path.is_file().then_some(path)
}

fn workspace_asset() -> Option<PathBuf> {
    // CARGO_MANIFEST_DIR is only set during `cargo run`/`cargo test`, never for installed binaries.
    let manifest_dir = option_env!("CARGO_MANIFEST_DIR")?;
    let path = PathBuf::from(manifest_dir)
        .join("assets")
        .join("models")
        .join(MODEL_FILENAME);
    path.is_file().then_some(path)
}

fn cache_path() -> Result<PathBuf, ModelError> {
    let base = dirs::cache_dir().ok_or(ModelError::NoCacheDir)?;
    Ok(base
        .join("oxidoc")
        .join("models")
        .join(MODEL_SHA256)
        .join(MODEL_FILENAME))
}

fn read_file(path: &std::path::Path) -> Result<Vec<u8>, ModelError> {
    fs::read(path).map_err(|source| ModelError::Read {
        path: path.display().to_string(),
        source,
    })
}

fn verify_sha(bytes: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());
    actual.eq_ignore_ascii_case(MODEL_SHA256)
}

fn download_to(dest: &std::path::Path) -> Result<(), ModelError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|source| ModelError::Read {
            path: parent.display().to_string(),
            source,
        })?;
    }

    eprintln!(
        "oxidoc: downloading search model (~17 MB) to {}",
        dest.display()
    );

    let mut response = ureq::get(MODEL_URL)
        .call()
        .map_err(|e| ModelError::Download(e.to_string()))?;

    let tmp = dest.with_extension("download");
    let mut writer = fs::File::create(&tmp).map_err(|source| ModelError::Read {
        path: tmp.display().to_string(),
        source,
    })?;

    let mut reader = response.body_mut().as_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut total: u64 = 0;
    let mut last_log = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| ModelError::Download(format!("read: {e}")))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .map_err(|source| ModelError::Read {
                path: tmp.display().to_string(),
                source,
            })?;
        total += n as u64;
        if total - last_log >= 4 * 1024 * 1024 {
            eprintln!("oxidoc: downloaded {} MB...", total / (1024 * 1024));
            last_log = total;
        }
    }
    writer.flush().ok();
    drop(writer);

    fs::rename(&tmp, dest).map_err(|source| ModelError::Read {
        path: dest.display().to_string(),
        source,
    })?;
    eprintln!("oxidoc: model cached ({} MB)", total / (1024 * 1024));
    Ok(())
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ModelError {
    #[error("could not locate a user cache directory")]
    NoCacheDir,

    #[error("downloaded model failed SHA-256 verification")]
    ChecksumMismatch,

    #[error("failed to download search model: {0}")]
    Download(String),

    #[error("failed to read {path}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
