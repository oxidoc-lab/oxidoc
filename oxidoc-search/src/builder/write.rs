//! Chunked lexical index files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::chunk::SortedChunk;
use crate::error::SearchError;
use crate::types::{ChunkEntry, ChunkManifest, LexicalIndex, Posting, SearchMetadata};

/// File name of the metadata file.
pub const META_FILE: &str = "search-meta.bin";

/// Returns the file name of chunk `id`.
pub fn chunk_file_name(id: u32) -> String {
    format!("search-chunk-{id}.bin")
}

/// Writes `index` into `dir` as chunked rkyv files.
///
/// Produces:
/// - `search-meta.bin`: [`SearchMetadata`] with the documents and the chunk manifest.
/// - `search-chunk-{id}.bin`: postings of the terms that share one 2-char prefix.
///
/// The same index produces the same bytes on every run. Chunk ids follow sorted
/// prefix order. Each chunk archives its terms in sorted order. `dir` must exist.
pub fn write_lexical(index: &LexicalIndex, dir: &Path) -> Result<(), SearchError> {
    let mut chunks: Vec<ChunkEntry> = Vec::new();

    for (chunk_id, (prefix, entries)) in partition_by_prefix(index).into_iter().enumerate() {
        let chunk_id = chunk_id as u32;
        let path = dir.join(chunk_file_name(chunk_id));
        let encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&SortedChunk::new(entries))
            .map_err(|e| encode_error(&path, e))?;
        write_file(path, &encoded)?;

        chunks.push(ChunkEntry {
            id: chunk_id,
            prefixes: vec![prefix],
        });
    }

    let metadata = SearchMetadata {
        documents: index.documents.clone(),
        manifest: ChunkManifest { chunks },
    };
    let path = dir.join(META_FILE);
    let encoded =
        rkyv::to_bytes::<rkyv::rancor::Error>(&metadata).map_err(|e| encode_error(&path, e))?;
    write_file(path, &encoded)
}

/// Groups postings by the first 2 chars of the term, in sorted prefix order.
fn partition_by_prefix(index: &LexicalIndex) -> BTreeMap<String, Vec<(&String, &Vec<Posting>)>> {
    let mut groups: BTreeMap<String, Vec<(&String, &Vec<Posting>)>> = BTreeMap::new();
    for (term, postings) in &index.postings {
        let prefix = if term.len() >= 2 && term.is_char_boundary(2) {
            term[..2].to_string()
        } else {
            term.chars().take(2).collect()
        };
        groups.entry(prefix).or_default().push((term, postings));
    }
    groups
}

fn encode_error(path: &Path, error: rkyv::rancor::Error) -> SearchError {
    SearchError::Encode {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

fn write_file(path: PathBuf, bytes: &[u8]) -> Result<(), SearchError> {
    std::fs::write(&path, bytes).map_err(|source| SearchError::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::build_lexical;
    use crate::record::{Field, Record};
    use crate::types::{ChunkPostings, DocMetadata};
    use std::collections::HashMap;

    /// Reads every file in `dir` into a map from file name to bytes.
    fn read_dir_bytes(dir: &Path) -> BTreeMap<String, Vec<u8>> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                let name = entry.file_name().to_string_lossy().into_owned();
                (name, std::fs::read(entry.path()).unwrap())
            })
            .collect()
    }

    #[test]
    fn write_lexical_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let mut postings = HashMap::new();
        postings.insert(
            "test".to_string(),
            vec![Posting {
                doc_id: 0,
                score: 0.95,
                positions: vec![0],
            }],
        );
        let index = LexicalIndex {
            postings,
            documents: vec![DocMetadata {
                id: 0,
                title: "Test".to_string(),
                path: "/test".to_string(),
                snippet: "test snippet".to_string(),
                text: String::new(),
                headings: vec![],
            }],
        };

        write_lexical(&index, tmp.path()).unwrap();

        let meta_bytes = std::fs::read(tmp.path().join(META_FILE)).unwrap();
        let metadata: SearchMetadata =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&meta_bytes).unwrap();
        assert_eq!(metadata.documents.len(), 1);
        assert!(!metadata.manifest.chunks.is_empty());

        let chunk_bytes = std::fs::read(tmp.path().join(chunk_file_name(0))).unwrap();
        let chunk: ChunkPostings =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&chunk_bytes).unwrap();
        assert!(chunk.contains_key("test"));
    }

    #[test]
    fn write_lexical_reports_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing");
        let index = build_lexical(&[]);
        match write_lexical(&index, &missing) {
            Err(SearchError::Write { path, .. }) => assert_eq!(path, missing.join(META_FILE)),
            other => panic!("expected a write error, got {other:?}"),
        }
    }

    #[test]
    fn write_lexical_is_deterministic() {
        // Many terms share a 2-char prefix, so each chunk holds several entries.
        let words: Vec<&str> = "configure compile compute contract convert collect combine \
            command commit compare search segment select server session setting render \
            request resolve return"
            .split_whitespace()
            .collect();
        let records: Vec<Record> = (0..6)
            .map(|i| {
                let text = words
                    .iter()
                    .cycle()
                    .skip(i * 3)
                    .take(words.len() + i)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut record = Record::new(
                    format!("page-{i}"),
                    format!("/page-{i}"),
                    format!("Page {i}"),
                );
                record.fields.push(Field::new("body", text));
                record
            })
            .collect();

        // Each build uses fresh std HashMaps with new random seeds.
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        write_lexical(&build_lexical(&records), first.path()).unwrap();
        write_lexical(&build_lexical(&records), second.path()).unwrap();

        let first_files = read_dir_bytes(first.path());
        let second_files = read_dir_bytes(second.path());
        assert!(first_files.contains_key(META_FILE));
        assert!(first_files.len() > 2, "expected several chunk files");
        assert_eq!(
            first_files.keys().collect::<Vec<_>>(),
            second_files.keys().collect::<Vec<_>>()
        );
        for (name, bytes) in &first_files {
            assert!(
                second_files[name] == *bytes,
                "{name} differs between two builds of the same records"
            );
        }
    }
}
