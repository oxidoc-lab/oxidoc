//! BM25 lexical index built from records.

use std::collections::HashMap;

use super::bm25::{compute_bm25, compute_idf};
use super::document::doc_metadata;
use crate::record::Record;
use crate::types::{DocMetadata, LexicalIndex, Posting};

/// Builds a BM25-scored inverted index from `records`.
///
/// Doc ids follow input order. Each body is tokenised with [`oxidoc_text::tokenize`],
/// the same pipeline the runtime searcher uses. Postings of a term are sorted by score
/// descending, then doc id ascending. Tags, boost and field weights do not change the
/// output.
pub fn build_lexical(records: &[Record]) -> LexicalIndex {
    let bodies: Vec<String> = records.iter().map(Record::body).collect();
    let tokenized_docs: Vec<Vec<String>> = bodies
        .iter()
        .map(String::as_str)
        .map(oxidoc_text::tokenize)
        .collect();

    // Document frequencies, term frequencies, and term positions.
    let mut df: HashMap<String, u32> = HashMap::new();
    let mut tf_lists: Vec<HashMap<String, u32>> = Vec::with_capacity(records.len());
    let mut position_lists: Vec<HashMap<String, Vec<u32>>> = Vec::with_capacity(records.len());

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

    let total_length: usize = tokenized_docs.iter().map(|d| d.len()).sum();
    let avg_doc_length = if records.is_empty() {
        0.0
    } else {
        total_length as f32 / records.len() as f32
    };

    let mut postings: HashMap<String, Vec<Posting>> = HashMap::new();

    for (doc_id, tf_map) in tf_lists.iter().enumerate() {
        let doc_length = tokenized_docs[doc_id].len() as f32;

        for (term, tf) in tf_map {
            let idf = compute_idf(*df.get(term).unwrap_or(&1), records.len() as u32);
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

    // Score descending, then doc id ascending, so the order is total.
    for postings_list in postings.values_mut() {
        postings_list.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });
    }

    let documents: Vec<DocMetadata> = records
        .iter()
        .zip(bodies)
        .enumerate()
        .map(|(idx, (record, body))| doc_metadata(idx as u32, record, body))
        .collect();

    LexicalIndex {
        postings,
        documents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Field;
    use crate::types::HeadingPos;
    use std::collections::BTreeMap;

    /// Builds the record the page path produces: one `body` field, url `/{slug}`.
    fn page_record(title: &str, slug: &str, text: &str) -> Record {
        let mut record = Record::new(slug, format!("/{slug}"), title);
        record.fields.push(Field::new("body", text));
        record
    }

    /// Serialises the postings in term order, for equality checks.
    fn postings_json(index: &LexicalIndex) -> String {
        let sorted: BTreeMap<&String, &Vec<Posting>> = index.postings.iter().collect();
        serde_json::to_string(&sorted).unwrap()
    }

    #[test]
    fn build_single_doc() {
        let index = build_lexical(&[page_record("Test", "test", "hello world")]);
        assert_eq!(index.documents.len(), 1);
        assert!(index.postings.contains_key("hello"));
        assert!(index.postings.contains_key("world"));
        assert_eq!(index.documents[0].id, 0);
        assert_eq!(index.documents[0].title, "Test");
        assert!(!index.postings["hello"][0].positions.is_empty());
    }

    #[test]
    fn build_multiple_docs() {
        let index = build_lexical(&[
            page_record("Doc1", "doc1", "hello world"),
            page_record("Doc2", "doc2", "hello there"),
        ]);
        assert_eq!(index.documents.len(), 2);
        let hello_postings = &index.postings["hello"];
        assert_eq!(hello_postings.len(), 2);
        assert!(hello_postings.iter().all(|p| p.score > 0.0));
    }

    #[test]
    fn build_empty() {
        let index = build_lexical(&[]);
        assert_eq!(index.documents.len(), 0);
        assert_eq!(index.postings.len(), 0);
    }

    #[test]
    fn postings_sorted_by_score_then_doc_id() {
        // Identical records give equal scores, so doc id breaks every tie.
        let records: Vec<Record> = (0..4)
            .map(|i| page_record(&format!("Doc{i}"), &format!("doc{i}"), "shared words here"))
            .collect();
        let index = build_lexical(&records);
        let term = oxidoc_text::tokenize("shared").remove(0);
        let ids: Vec<u32> = index.postings[&term].iter().map(|p| p.doc_id).collect();
        assert_eq!(ids, vec![0, 1, 2, 3]);
    }

    #[test]
    fn page_record_matches_page_metadata() {
        let text = "Intro text. Setup content here.";
        let mut record = page_record("Guide", "docs/guide", text);
        record.headings.push(HeadingPos {
            title: "Setup".to_string(),
            anchor: "setup".to_string(),
            depth: 2,
            offset: 12,
        });
        let index = build_lexical(&[record]);
        let doc = &index.documents[0];
        assert_eq!(doc.path, "/docs/guide");
        assert_eq!(doc.text, text);
        assert_eq!(doc.snippet, text);
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].offset, 12);
    }

    #[test]
    fn tags_boost_and_weight_do_not_change_output() {
        let plain = vec![
            page_record("One", "one", "compile the project and run it"),
            page_record("Two", "two", "run the server with a config"),
        ];
        let mut decorated = plain.clone();
        for record in &mut decorated {
            record
                .tags
                .push(("section".to_string(), "guide".to_string()));
            record.boost = 3.0;
            record.fields[0].weight = 2.5;
        }

        let a = build_lexical(&plain);
        let b = build_lexical(&decorated);
        assert_eq!(postings_json(&a), postings_json(&b));
        assert_eq!(
            serde_json::to_string(&a.documents).unwrap(),
            serde_json::to_string(&b.documents).unwrap()
        );
    }
}
