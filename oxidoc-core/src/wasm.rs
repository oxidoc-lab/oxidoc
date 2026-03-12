use crate::error::{OxidocError, Result};
use std::path::Path;
use std::process::Command;

/// Wasm crates to build, in order: (crate name, output file stem)
const WASM_CRATES: &[(&str, &str)] = &[
    ("oxidoc-registry", "oxidoc_registry"),
    ("oxidoc-openapi", "oxidoc_openapi"),
    ("oxidoc-search", "oxidoc_search"),
];

/// Build wasm crates and run wasm-bindgen, writing output to `output_dir`.
///
/// Locates the oxidoc workspace by finding the `oxidoc-core` crate's manifest,
/// then builds each wasm crate with `cargo build --target wasm32-unknown-unknown --release`
/// and processes with `wasm-bindgen --target web`.
pub fn build_wasm(output_dir: &Path) -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let target_dir = resolve_target_dir(&workspace_root)?;
    let wasm_target_dir = target_dir.join("wasm32-unknown-unknown/release");

    // Build all wasm crates in one cargo invocation
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release");
    for (crate_name, _) in WASM_CRATES {
        cmd.arg("-p").arg(*crate_name);
    }
    cmd.current_dir(&workspace_root);

    tracing::info!("Building wasm crates...");
    let status = cmd.status().map_err(|e| OxidocError::WasmBuild {
        message: format!("Failed to run cargo: {e}"),
    })?;
    if !status.success() {
        return Err(OxidocError::WasmBuild {
            message: format!("cargo build failed with {status}"),
        });
    }

    // Run wasm-bindgen on each output
    for (_, stem) in WASM_CRATES {
        let wasm_file = wasm_target_dir.join(format!("{stem}.wasm"));
        if !wasm_file.exists() {
            return Err(OxidocError::WasmBuild {
                message: format!("Expected wasm file not found: {}", wasm_file.display()),
            });
        }

        let status = Command::new("wasm-bindgen")
            .arg(&wasm_file)
            .arg("--out-dir")
            .arg(output_dir)
            .arg("--target")
            .arg("web")
            .arg("--no-typescript")
            .status()
            .map_err(|e| OxidocError::WasmBuild {
                message: format!("Failed to run wasm-bindgen: {e}. Is it installed? Run: cargo install wasm-bindgen-cli"),
            })?;
        if !status.success() {
            return Err(OxidocError::WasmBuild {
                message: format!("wasm-bindgen failed for {stem}"),
            });
        }
    }

    tracing::info!("Wasm build complete");
    Ok(())
}

/// Find the oxidoc workspace root by looking for the workspace Cargo.toml.
fn find_workspace_root() -> Result<std::path::PathBuf> {
    // Use cargo to locate the workspace root
    let output = Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .map_err(|e| OxidocError::WasmBuild {
            message: format!("Failed to locate workspace: {e}"),
        })?;

    if !output.status.success() {
        return Err(OxidocError::WasmBuild {
            message: "Failed to locate cargo workspace".to_string(),
        });
    }

    let manifest_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let workspace_root = Path::new(&manifest_path)
        .parent()
        .ok_or_else(|| OxidocError::WasmBuild {
            message: "Could not determine workspace root".to_string(),
        })?
        .to_path_buf();

    Ok(workspace_root)
}

/// Resolve the target directory (respects CARGO_TARGET_DIR and .cargo/config.toml).
fn resolve_target_dir(workspace_root: &Path) -> Result<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        return Ok(Path::new(&dir).to_path_buf());
    }
    Ok(workspace_root.join("target"))
}
