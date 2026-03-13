pub mod extract;
pub mod lexical;
pub mod semantic;
pub mod types;

pub use types::*;

use crate::config::SearchConfig;
use crate::crawler::NavGroup;
use crate::error::Result;
use std::path::Path;

/// Generate both lexical and optional semantic search indices for the site.
pub fn generate_search_index(
    nav_groups: &[NavGroup],
    output_dir: &Path,
    config: &SearchConfig,
) -> Result<()> {
    // Extract searchable text from all pages.
    let pages = extract::extract_page_text(nav_groups)?;

    // Always generate lexical index.
    let lexical = lexical::build_lexical_index(&pages);
    lexical::write_lexical_index(&lexical, output_dir)?;
    tracing::info!(
        terms = lexical.postings.len(),
        docs = lexical.documents.len(),
        "Lexical index generated"
    );

    // Optionally generate semantic index with pre-computed embeddings.
    if config.model_path.is_some() {
        match semantic::build_vector_index(&pages, config) {
            Ok(Some(vectors)) => {
                semantic::write_vector_index(&vectors, output_dir)?;
                tracing::info!(
                    docs = vectors.documents.len(),
                    dim = vectors.dimension,
                    "Vector index generated"
                );
                // Copy model to output dir so the browser can fetch it for query embedding.
                if let Err(e) = semantic::copy_model_to_output(config, output_dir) {
                    tracing::warn!("Failed to copy model to output: {}", e);
                }
            }
            Ok(None) => {
                tracing::info!("Semantic indexing skipped (no model configured)");
            }
            Err(e) => {
                tracing::warn!("Semantic indexing failed: {}", e);
                // Don't fail the build — lexical search is the fallback.
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_search_index_full_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            docs.join("intro.rdx"),
            "# Introduction\n\nWelcome to the documentation.",
        )
        .unwrap();
        std::fs::write(
            docs.join("guide.rdx"),
            "# User Guide\n\nThis is the user guide with helpful information.",
        )
        .unwrap();

        let nav_groups = vec![NavGroup {
            title: "Docs".to_string(),
            pages: vec![
                crate::crawler::PageEntry {
                    title: "Introduction".to_string(),
                    slug: "intro".to_string(),
                    file_path: docs.join("intro.rdx"),
                    group: None,
                },
                crate::crawler::PageEntry {
                    title: "User Guide".to_string(),
                    slug: "guide".to_string(),
                    file_path: docs.join("guide.rdx"),
                    group: None,
                },
            ],
        }];

        let output = tmp.path().join("output");
        std::fs::create_dir(&output).unwrap();

        let config = SearchConfig::default();

        generate_search_index(&nav_groups, &output, &config).unwrap();

        // Verify chunked index was created (metadata + at least one chunk).
        let meta_path = output.join("search-meta.bin");
        assert!(meta_path.exists());

        let meta_bytes = std::fs::read(&meta_path).unwrap();
        let metadata: types::SearchMetadata =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&meta_bytes)
                .expect("SearchMetadata deserialization failed");
        assert_eq!(metadata.documents.len(), 2);
        assert!(!metadata.manifest.chunks.is_empty());

        // Vector index should not be created (no model).
        let vector_path = output.join("search-vectors.json");
        assert!(!vector_path.exists());
    }
}
