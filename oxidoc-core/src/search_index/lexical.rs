use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

use super::types::{self, DocMetadata, LexicalIndex, PageContent, Posting};

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

    // Tokenize all documents and compute statistics.
    let tokenized_docs: Vec<Vec<String>> = pages.iter().map(|page| tokenize(&page.text)).collect();

    // Compute document frequencies and term frequencies.
    let mut df: HashMap<String, u32> = HashMap::new();
    let mut tf_lists: Vec<HashMap<String, u32>> = Vec::new();

    for tokens in &tokenized_docs {
        let mut tf: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *tf.entry(token.clone()).or_insert(0) += 1;
        }

        for term in tf.keys() {
            *df.entry(term.clone()).or_insert(0) += 1;
        }

        tf_lists.push(tf);
    }

    // Compute average document length.
    let total_length: usize = tokenized_docs.iter().map(|d| d.len()).sum();
    let avg_doc_length = if pages.is_empty() {
        0.0
    } else {
        total_length as f32 / pages.len() as f32
    };

    // Build inverted index with BM25 scores.
    let mut postings: HashMap<String, Vec<Posting>> = HashMap::new();

    for (doc_id, tf_map) in tf_lists.iter().enumerate() {
        let doc_length = tokenized_docs[doc_id].len() as f32;

        for (term, tf) in tf_map {
            let idf = compute_idf(*df.get(term).unwrap_or(&1), pages.len() as u32);
            let score = compute_bm25(idf, *tf as f32, doc_length, avg_doc_length);

            postings.entry(term.clone()).or_default().push(Posting {
                doc_id: doc_id as u32,
                score,
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

/// Tokenize text into lowercase terms.
///
/// Splits on whitespace, strips non-alphanumeric chars (preserving hyphens and underscores),
/// splits camelCase/PascalCase into sub-words (emitting both compound and parts),
/// and filters tokens shorter than 2 characters. This logic must match the tokenizer in
/// `oxidoc-search/src/lexical.rs` to ensure consistent indexing and querying.
fn tokenize(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for word in text.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        let lower = cleaned.to_lowercase();
        if lower.len() <= 1 {
            continue;
        }
        // Split camelCase/PascalCase into sub-words
        let parts = split_camel_case(&cleaned);
        if parts.len() > 1 {
            // Emit the compound token
            result.push(lower);
            // Emit each sub-word
            for part in parts {
                let p = part.to_lowercase();
                if p.len() > 1 {
                    result.push(p);
                }
            }
        } else {
            result.push(lower);
        }
    }
    result
}

/// Split a camelCase or PascalCase string into its component words.
/// e.g. "CodeBlock" -> ["Code", "Block"], "myFunc" -> ["my", "Func"]
fn split_camel_case(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = s.as_bytes();
    let mut start = 0;
    for i in 1..bytes.len() {
        let curr_upper = bytes[i].is_ascii_uppercase();
        let prev_upper = bytes[i - 1].is_ascii_uppercase();
        // Split at lowercase->uppercase boundary (e.g. "code|Block")
        // Split at uppercase->uppercase->lowercase boundary (e.g. "XML|Parser")
        if curr_upper && (!prev_upper || (i + 1 < bytes.len() && bytes[i + 1].is_ascii_lowercase()))
        {
            parts.push(&s[start..i]);
            start = i;
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
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

/// Write the lexical index to a JSON file.
pub fn write_lexical_index(index: &LexicalIndex, output_dir: &Path) -> Result<()> {
    let output_path = output_dir.join("search-lexical.json");
    let json = serde_json::to_string(index).map_err(|e| crate::error::OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: std::io::Error::other(e),
    })?;
    std::fs::write(&output_path, json).map_err(|e| crate::error::OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokens = tokenize("Hello, world! How are you?");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"are".to_string()));
    }

    #[test]
    fn test_tokenize_preserves_hyphens() {
        let tokens = tokenize("rust-lang is great");
        assert!(tokens.contains(&"rust-lang".to_string()));
    }

    #[test]
    fn test_tokenize_filters_short_tokens() {
        let tokens = tokenize("a b cd efgh");
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"b".to_string()));
        assert!(tokens.contains(&"cd".to_string()));
        assert!(tokens.contains(&"efgh".to_string()));
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
        assert!(tmp.path().join("search-lexical.json").exists());

        let json = std::fs::read_to_string(tmp.path().join("search-lexical.json")).unwrap();
        let deserialized: LexicalIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.documents.len(), 1);
        assert_eq!(deserialized.postings.len(), 1);
    }
}
