//! Lexical index for site pages, built by `oxidoc_search::builder`.

use crate::error::Result;
use oxidoc_search::Record;
use std::path::Path;

use super::types::{LexicalIndex, PageContent};

/// Builds a BM25-scored lexical index from page content. Doc ids follow page order.
pub fn build_lexical_index(pages: &[PageContent]) -> LexicalIndex {
    let records: Vec<Record> = pages.iter().map(Record::from).collect();
    oxidoc_search::builder::build_lexical(&records)
}

/// Writes the lexical index into `output_dir` as `search-meta.bin` plus chunk files.
///
/// The same index produces the same bytes on every run.
pub fn write_lexical_index(index: &LexicalIndex, output_dir: &Path) -> Result<()> {
    oxidoc_search::builder::write_lexical(index, output_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_path_is_slash_slug() {
        let pages = vec![PageContent {
            title: "Test".to_string(),
            slug: "docs/test".to_string(),
            text: "hello world".to_string(),
            headings: vec![],
        }];
        let index = build_lexical_index(&pages);
        assert_eq!(index.documents[0].path, "/docs/test");
        assert_eq!(index.documents[0].text, "hello world");
        assert!(index.postings.contains_key("hello"));
    }

    #[test]
    fn write_error_maps_to_file_write() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing");
        let err = write_lexical_index(&build_lexical_index(&[]), &missing).unwrap_err();
        match err {
            crate::error::OxidocError::FileWrite { path, .. } => {
                assert_eq!(path, missing.join("search-meta.bin").display().to_string());
            }
            other => panic!("expected FileWrite, got {other:?}"),
        }
    }
}
