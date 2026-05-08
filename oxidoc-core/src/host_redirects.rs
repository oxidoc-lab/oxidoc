//! Per-host config files that upgrade trailing-slash handling from a meta-refresh
//! stub (`<slug>/index.html`) to a real HTTP 301 on hosts that read them.
//!
//! Hosts without a config layer (GitHub Pages, plain S3, raw nginx) still rely on
//! the meta-refresh stubs emitted by `builder::folder_index`. Hosts that *do* read
//! these files will short-circuit at the edge before the stub is even fetched.

use crate::error::{OxidocError, Result};
use std::path::Path;

/// Netlify / Cloudflare Pages `_redirects` rule. The `301!` force flag ensures the
/// redirect wins over any shadowing static file (Netlify only — Cloudflare ignores
/// `!` and treats it as a regular 301, which is also fine).
const NETLIFY_REDIRECTS: &str = "/*/  /:splat  301!\n";

/// Vercel `vercel.json` config — `:path+` matches one-or-more URL segments so this
/// only fires for non-root trailing-slash URLs.
const VERCEL_REDIRECTS: &str = r#"{
  "redirects": [
    {
      "source": "/:path+/",
      "destination": "/:path+",
      "permanent": true
    }
  ]
}
"#;

/// Write `_redirects` and `vercel.json` to the build output. Skips writing when a
/// file already exists at the target so user-authored config is never overwritten.
pub fn generate_host_redirects(output_dir: &Path) -> Result<()> {
    write_if_absent(&output_dir.join("_redirects"), NETLIFY_REDIRECTS)?;
    write_if_absent(&output_dir.join("vercel.json"), VERCEL_REDIRECTS)?;
    Ok(())
}

fn write_if_absent(path: &Path, content: &str) -> Result<()> {
    if path.is_file() {
        return Ok(());
    }
    std::fs::write(path, content).map_err(|e| OxidocError::FileWrite {
        path: path.display().to_string(),
        source: e,
    })
}
