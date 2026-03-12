use crate::error::{OxidocError, Result};
use std::path::Path;

/// Copy static assets from `public/` and `docs/` (non-.rdx files) to the output directory.
pub fn copy_assets(project_root: &Path, output_dir: &Path) -> Result<usize> {
    let mut count = 0;

    // Copy everything from public/ directory (if it exists)
    let public_dir = project_root.join("public");
    if public_dir.is_dir() {
        count += copy_dir_recursive(&public_dir, output_dir)?;
    }

    // Copy assets/ directory (logo, images, etc.)
    let assets_dir = project_root.join("assets");
    if assets_dir.is_dir() {
        let dst = output_dir.join("assets");
        std::fs::create_dir_all(&dst).map_err(|e| OxidocError::DirCreate {
            path: dst.display().to_string(),
            source: e,
        })?;
        count += copy_dir_recursive(&assets_dir, &dst)?;
    }

    // Copy non-.rdx files from docs/ (images, etc.)
    let docs_dir = project_root.join("docs");
    if docs_dir.is_dir() {
        count += copy_non_rdx(&docs_dir, &docs_dir, output_dir)?;
    }

    Ok(count)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize> {
    let mut count = 0;
    let entries = std::fs::read_dir(src).map_err(|e| OxidocError::FileRead {
        path: src.display().to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| OxidocError::FileRead {
            path: src.display().to_string(),
            source: e,
        })?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| OxidocError::DirCreate {
                path: dst_path.display().to_string(),
                source: e,
            })?;
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| OxidocError::FileWrite {
                path: dst_path.display().to_string(),
                source: std::io::Error::new(e.kind(), e.to_string()),
            })?;
            count += 1;
        }
    }

    Ok(count)
}

fn copy_non_rdx(dir: &Path, docs_root: &Path, output_dir: &Path) -> Result<usize> {
    let mut count = 0;
    let entries = std::fs::read_dir(dir).map_err(|e| OxidocError::FileRead {
        path: dir.display().to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| OxidocError::FileRead {
            path: dir.display().to_string(),
            source: e,
        })?;
        let src_path = entry.path();

        if src_path.is_dir() {
            count += copy_non_rdx(&src_path, docs_root, output_dir)?;
        } else if src_path.extension().and_then(|e| e.to_str()) != Some("rdx") {
            let relative =
                src_path
                    .strip_prefix(docs_root)
                    .map_err(|_| OxidocError::PathNotUnderRoot {
                        path: src_path.display().to_string(),
                        root: docs_root.display().to_string(),
                    })?;
            let dst_path = output_dir.join(relative);

            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }

            std::fs::copy(&src_path, &dst_path).map_err(|e| OxidocError::FileWrite {
                path: dst_path.display().to_string(),
                source: std::io::Error::new(e.kind(), e.to_string()),
            })?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_public_assets() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let public = root.join("public");
        let images = public.join("images");
        std::fs::create_dir_all(&images).unwrap();
        std::fs::write(public.join("favicon.ico"), "icon").unwrap();
        std::fs::write(images.join("logo.png"), "png").unwrap();

        let output = root.join("dist");
        std::fs::create_dir(&output).unwrap();
        let count = copy_assets(root, &output).unwrap();

        assert_eq!(count, 2);
        assert!(output.join("favicon.ico").exists());
        assert!(output.join("images/logo.png").exists());
    }

    #[test]
    fn copy_docs_non_rdx() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(docs.join("intro.rdx"), "# Intro").unwrap();
        std::fs::write(docs.join("diagram.png"), "png").unwrap();

        let output = root.join("dist");
        std::fs::create_dir(&output).unwrap();
        let count = copy_assets(root, &output).unwrap();

        assert_eq!(count, 1);
        assert!(output.join("diagram.png").exists());
        assert!(!output.join("intro.rdx").exists());
    }

    #[test]
    fn no_public_dir_is_fine() {
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path().join("dist");
        std::fs::create_dir(&output).unwrap();
        let count = copy_assets(tmp.path(), &output).unwrap();
        assert_eq!(count, 0);
    }
}
