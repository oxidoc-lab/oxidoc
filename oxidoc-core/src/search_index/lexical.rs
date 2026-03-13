use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

use super::types::{
    self, ChunkEntry, ChunkManifest, DocMetadata, LexicalIndex, PageContent, Posting,
    SearchMetadata,
};

/// BM25 parameters (standard tuning).
const K1: f32 = 1.2;
const B: f32 = 0.75;

/// Build a BM25-scored lexical (inverted) index from page content.
pub fn build_lexical_index(pages: &[PageContent]) -> LexicalIndex {
    // Build document metadata.
    let documents: Vec<DocMetadata> = pages
        .iter()
        .enumerate()
        .map(|(idx, page)| DocMetadata {
            id: idx as u32,
            title: page.title.clone(),
            path: format!("/{}", page.slug),
            snippet: types::create_snippet(&page.text, 160),
            text: page.text.clone(),
            headings: page.headings.clone(),
        })
        .collect();

    // Tokenize all documents and compute statistics + positions.
    let tokenized_docs: Vec<Vec<String>> = pages
        .iter()
        .map(|page| oxidoc_text::tokenize(&page.text))
        .collect();

    // Compute document frequencies, term frequencies, and term positions.
    let mut df: HashMap<String, u32> = HashMap::new();
    let mut tf_lists: Vec<HashMap<String, u32>> = Vec::new();
    let mut position_lists: Vec<HashMap<String, Vec<u32>>> = Vec::new();

    for tokens in &tokenized_docs {
        let mut tf: HashMap<String, u32> = HashMap::new();
        let mut positions: HashMap<String, Vec<u32>> = HashMap::new();

        for (pos, token) in tokens.iter().enumerate() {
            *tf.entry(token.clone()).or_insert(0) += 1;
            positions.entry(token.clone()).or_default().push(pos as u32);
        }

        for term in tf.keys() {
            *df.entry(term.clone()).or_insert(0) += 1;
        }

        tf_lists.push(tf);
        position_lists.push(positions);
    }

    // Compute average document length.
    let total_length: usize = tokenized_docs.iter().map(|d| d.len()).sum();
    let avg_doc_length = if pages.is_empty() {
        0.0
    } else {
        total_length as f32 / pages.len() as f32
    };

    // Build inverted index with BM25 scores and positions.
    let mut postings: HashMap<String, Vec<Posting>> = HashMap::new();

    for (doc_id, tf_map) in tf_lists.iter().enumerate() {
        let doc_length = tokenized_docs[doc_id].len() as f32;

        for (term, tf) in tf_map {
            let idf = compute_idf(*df.get(term).unwrap_or(&1), pages.len() as u32);
            let score = compute_bm25(idf, *tf as f32, doc_length, avg_doc_length);
            let positions = position_lists[doc_id]
                .get(term)
                .cloned()
                .unwrap_or_default();

            postings.entry(term.clone()).or_default().push(Posting {
                doc_id: doc_id as u32,
                score,
                positions,
            });
        }
    }

    // Sort postings by score (descending) for faster retrieval.
    for postings_list in postings.values_mut() {
        postings_list.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    LexicalIndex {
        postings,
        documents,
    }
}

/// Compute inverse document frequency.
fn compute_idf(df: u32, total_docs: u32) -> f32 {
    let df = (df as f32).max(1.0);
    ((total_docs as f32 - df + 0.5) / (df + 0.5) + 1.0).ln()
}

/// Compute BM25 score.
fn compute_bm25(idf: f32, tf: f32, doc_length: f32, avg_doc_length: f32) -> f32 {
    let normalizer = 1.0 - B + B * (doc_length / avg_doc_length);
    idf * ((tf * (K1 + 1.0)) / (tf + K1 * normalizer))
}

/// Write the lexical index as chunked binary files.
///
/// Produces:
/// - `search-meta.bin`: SearchMetadata (documents + chunk manifest)
/// - `search-chunk-{id}.bin`: postings for each chunk (partitioned by 2-char prefix)
pub fn write_lexical_index(index: &LexicalIndex, output_dir: &Path) -> Result<()> {
    // Partition postings by first 2 chars of key.
    let mut prefix_groups: HashMap<String, HashMap<String, Vec<Posting>>> = HashMap::new();
    for (key, postings) in &index.postings {
        let prefix = if key.len() >= 2 {
            key[..2].to_string()
        } else {
            key.to_string()
        };
        prefix_groups
            .entry(prefix)
            .or_default()
            .insert(key.clone(), postings.clone());
    }

    // Assign chunk IDs and write chunk files.
    let mut chunks: Vec<ChunkEntry> = Vec::new();
    let mut sorted_prefixes: Vec<String> = prefix_groups.keys().cloned().collect();
    sorted_prefixes.sort();

    for (chunk_id, prefix) in sorted_prefixes.iter().enumerate() {
        let chunk_postings = prefix_groups.remove(prefix).unwrap_or_default();
        let chunk_path = output_dir.join(format!("search-chunk-{}.bin", chunk_id));
        let encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&chunk_postings).map_err(|e| {
            crate::error::OxidocError::FileWrite {
                path: chunk_path.display().to_string(),
                source: std::io::Error::other(e),
            }
        })?;
        std::fs::write(&chunk_path, encoded).map_err(|e| crate::error::OxidocError::FileWrite {
            path: chunk_path.display().to_string(),
            source: e,
        })?;

        chunks.push(ChunkEntry {
            id: chunk_id as u32,
            prefixes: vec![prefix.clone()],
        });
    }

    // Write metadata file.
    let metadata = SearchMetadata {
        documents: index.documents.clone(),
        manifest: ChunkManifest { chunks },
    };
    let meta_path = output_dir.join("search-meta.bin");
    let meta_encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata).map_err(|e| {
        crate::error::OxidocError::FileWrite {
            path: meta_path.display().to_string(),
            source: std::io::Error::other(e),
        }
    })?;
    std::fs::write(&meta_path, meta_encoded).map_err(|e| crate::error::OxidocError::FileWrite {
        path: meta_path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = oxidoc_text::tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokens = oxidoc_text::tokenize("Hello, world! How are you?");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_tokenize_preserves_hyphens() {
        let tokens = oxidoc_text::tokenize("rust-lang is great");
        assert!(tokens.contains(&"rust-lang".to_string()));
    }

    #[test]
    fn test_tokenize_filters_short_tokens() {
        let tokens = oxidoc_text::tokenize("a b cd efgh");
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"b".to_string()));
    }

    #[test]
    fn test_tokenize_stemming() {
        let tokens = oxidoc_text::tokenize("running documentation");
        assert!(tokens.contains(&"run".to_string()));
        assert!(tokens.contains(&"document".to_string()));
    }

    #[test]
    fn test_tokenize_stop_words_filtered() {
        let tokens = oxidoc_text::tokenize("the quick and brown fox");
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"and".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
    }

    #[test]
    fn test_compute_idf() {
        let idf1 = compute_idf(1, 100);
        let idf2 = compute_idf(50, 100);
        assert!(idf1 > idf2, "Rare terms should have higher IDF");
    }

    #[test]
    fn test_build_lexical_index_single_doc() {
        let pages = vec![PageContent {
            title: "Test".to_string(),
            slug: "test".to_string(),
            text: "hello world".to_string(),
            headings: vec![],
        }];

        let index = build_lexical_index(&pages);
        assert_eq!(index.documents.len(), 1);
        assert!(index.postings.contains_key("hello"));
        assert!(index.postings.contains_key("world"));
        assert_eq!(index.documents[0].id, 0);
        assert_eq!(index.documents[0].title, "Test");
        // Verify positions are recorded
        let hello_posting = &index.postings["hello"][0];
        assert!(!hello_posting.positions.is_empty());
    }

    #[test]
    fn test_build_lexical_index_multiple_docs() {
        let pages = vec![
            PageContent {
                title: "Doc1".to_string(),
                slug: "doc1".to_string(),
                text: "hello world".to_string(),
                headings: vec![],
            },
            PageContent {
                title: "Doc2".to_string(),
                slug: "doc2".to_string(),
                text: "hello there".to_string(),
                headings: vec![],
            },
        ];

        let index = build_lexical_index(&pages);
        assert_eq!(index.documents.len(), 2);
        let hello_postings = &index.postings["hello"];
        assert_eq!(hello_postings.len(), 2);
        assert!(hello_postings.iter().all(|p| p.score > 0.0));
    }

    #[test]
    fn test_build_lexical_index_empty() {
        let pages: Vec<PageContent> = Vec::new();
        let index = build_lexical_index(&pages);
        assert_eq!(index.documents.len(), 0);
        assert_eq!(index.postings.len(), 0);
    }

    #[test]
    fn test_write_lexical_index() {
        let tmp = tempfile::tempdir().unwrap();
        let index = LexicalIndex {
            postings: {
                let mut m = HashMap::new();
                m.insert(
                    "test".to_string(),
                    vec![Posting {
                        doc_id: 0,
                        score: 0.95,
                        positions: vec![0],
                    }],
                );
                m
            },
            documents: vec![DocMetadata {
                id: 0,
                title: "Test".to_string(),
                path: "/test".to_string(),
                snippet: "test snippet".to_string(),
                text: String::new(),
                headings: vec![],
            }],
        };

        write_lexical_index(&index, tmp.path()).unwrap();
        // Should produce metadata and chunk files
        assert!(tmp.path().join("search-meta.bin").exists());

        // Verify metadata can be deserialized
        let meta_bytes = std::fs::read(tmp.path().join("search-meta.bin")).unwrap();
        let metadata: SearchMetadata = rkyv::from_bytes::<_, rkyv::rancor::Error>(&meta_bytes)
            .expect("SearchMetadata deserialization failed");
        assert_eq!(metadata.documents.len(), 1);
        assert!(!metadata.manifest.chunks.is_empty());

        // Verify chunk can be deserialized
        let chunk_path = tmp.path().join("search-chunk-0.bin");
        assert!(chunk_path.exists());
        let chunk_bytes = std::fs::read(&chunk_path).unwrap();
        let chunk: HashMap<String, Vec<Posting>> =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&chunk_bytes)
                .expect("ChunkPostings deserialization failed");
        assert!(chunk.contains_key("test"));
    }
}
