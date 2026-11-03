//! Chunked lexical index files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::chunk::SortedChunk;
use super::records::encode_records_file;
use crate::error::SearchError;
use crate::records::{RECORDS_FILE, SearchRecords};
use crate::types::{ChunkEntry, ChunkManifest, LexicalIndex, Posting, SearchMetadata};

/// File name of the metadata file.
pub const META_FILE: &str = "search-meta.bin";

/// Returns the file name of chunk `id`.
pub fn chunk_file_name(id: u32) -> String {
    format!("search-chunk-{id}.bin")
}

/// One encoded index file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexFile {
    /// File name, such as `search-meta.bin`. It holds no directory part.
    pub name: String,
    /// Encoded file contents.
    pub bytes: Vec<u8>,
}

/// Encodes `index` as chunked rkyv files in memory.
///
/// Returns, in this order:
/// - `search-chunk-{id}.bin`: postings of the terms that share one 2-char prefix.
/// - `search-meta.bin`: [`SearchMetadata`] with the documents and the chunk manifest.
/// - `search-records.bin`: `records`, only when [`SearchRecords::has_extras`] is true.
///
/// `records` comes from [`build_records`](super::build_records) over the same input
/// as `index`. The same input produces the same bytes on every run. Chunk ids follow
/// sorted prefix order. Each chunk archives its terms in sorted order.
/// An [`SearchError::Encode`] error names the file without a directory.
pub fn encode_lexical(
    index: &LexicalIndex,
    records: Option<&SearchRecords>,
) -> Result<Vec<IndexFile>, SearchError> {
    let mut files: Vec<IndexFile> = Vec::new();
    let mut chunks: Vec<ChunkEntry> = Vec::new();

    for (chunk_id, (prefix, entries)) in partition_by_prefix(index).into_iter().enumerate() {
        let chunk_id = chunk_id as u32;
        let name = chunk_file_name(chunk_id);
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&SortedChunk::new(entries))
            .map_err(|e| encode_error(&name, e))?
            .to_vec();
        files.push(IndexFile { name, bytes });

        chunks.push(ChunkEntry {
            id: chunk_id,
            prefixes: vec![prefix],
        });
    }

    let metadata = SearchMetadata {
        documents: index.documents.clone(),
        manifest: ChunkManifest { chunks },
    };
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata)
        .map_err(|e| encode_error(META_FILE, e))?
        .to_vec();
    files.push(IndexFile {
        name: META_FILE.to_string(),
        bytes,
    });
    if let Some(bytes) = encode_records_file(records, index.documents.len())? {
        files.push(IndexFile {
            name: RECORDS_FILE.to_string(),
            bytes,
        });
    }
    Ok(files)
}

/// Writes `index` into `dir` as chunked rkyv files.
///
/// Writes the files of [`encode_lexical`], in its order. Without a records file,
/// it removes a `search-records.bin` from an earlier build. `dir` must exist.
pub fn write_lexical(
    index: &LexicalIndex,
    records: Option<&SearchRecords>,
    dir: &Path,
) -> Result<(), SearchError> {
    let files = encode_lexical(index, records).map_err(|error| match error {
        SearchError::Encode { path, message } => SearchError::Encode {
            path: dir.join(path),
            message,
        },
        other => other,
    })?;
    let mut has_records = false;
    for file in files {
        has_records |= file.name == RECORDS_FILE;
        write_file(dir.join(&file.name), &file.bytes)?;
    }
    if has_records {
        return Ok(());
    }
    let path = dir.join(RECORDS_FILE);
    match std::fs::remove_file(&path) {
        Err(source) if source.kind() != std::io::ErrorKind::NotFound => {
            Err(SearchError::Write { path, source })
        }
        _ => Ok(()),
    }
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

fn encode_error(name: &str, error: rkyv::rancor::Error) -> SearchError {
    SearchError::Encode {
        path: PathBuf::from(name),
        message: error.to_string(),
    }
}

/// Writes `bytes` to `path`.
fn write_file(path: PathBuf, bytes: &[u8]) -> Result<(), SearchError> {
    std::fs::write(&path, bytes).map_err(|source| SearchError::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{build_lexical, build_records};
    use crate::record::{Field, Record};
    use crate::records::{RECORDS_FILE, decode_records};
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

        write_lexical(&index, None, tmp.path()).unwrap();

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
        match write_lexical(&index, None, &missing) {
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
        let extras = build_records(&records);
        write_lexical(&build_lexical(&records), Some(&extras), first.path()).unwrap();
        write_lexical(&build_lexical(&records), Some(&extras), second.path()).unwrap();

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

    /// Builds a two-record input and applies `extra` to the first record.
    fn records_with(extra: fn(&mut Record)) -> Vec<Record> {
        let mut records: Vec<Record> = ["alpha text", "beta text"]
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let mut record = Record::new(format!("r{i}"), format!("/r{i}"), format!("R{i}"));
                record.fields.push(Field::new("body", *text));
                record
            })
            .collect();
        extra(&mut records[0]);
        records
    }

    /// One case: a label, an edit to the first record, and whether the file is written.
    type ExtrasCase = (&'static str, fn(&mut Record), bool);

    #[test]
    fn records_file_written_only_with_extras() {
        let cases: [ExtrasCase; 5] = [
            ("page only", |_| {}, false),
            (
                "tag",
                |r| r.tags.push(("kind".to_string(), "library".to_string())),
                true,
            ),
            ("boost", |r| r.boost = 1.2, true),
            (
                "two fields",
                |r| r.fields.push(Field::new("readme", "more")),
                true,
            ),
            ("weight", |r| r.fields[0].weight = 3.0, true),
        ];
        for (name, extra, want) in cases {
            let records = records_with(extra);
            let tmp = tempfile::tempdir().unwrap();
            let index = build_lexical(&records);
            write_lexical(&index, Some(&build_records(&records)), tmp.path()).unwrap();
            let path = tmp.path().join(RECORDS_FILE);
            assert_eq!(path.exists(), want, "{name}");
            if want {
                let decoded = decode_records(&std::fs::read(&path).unwrap()).unwrap();
                assert_eq!(decoded, build_records(&records), "{name}");
            }
        }
    }

    #[test]
    fn page_only_build_removes_stale_records_file() {
        let tmp = tempfile::tempdir().unwrap();
        let tagged = records_with(|r| r.boost = 1.2);
        write_lexical(
            &build_lexical(&tagged),
            Some(&build_records(&tagged)),
            tmp.path(),
        )
        .unwrap();
        assert!(tmp.path().join(RECORDS_FILE).exists());

        let plain = records_with(|_| {});
        write_lexical(
            &build_lexical(&plain),
            Some(&build_records(&plain)),
            tmp.path(),
        )
        .unwrap();
        assert!(!tmp.path().join(RECORDS_FILE).exists());
    }

    #[test]
    fn encode_lexical_matches_written_files() {
        let records = records_with(|r| r.boost = 1.2);
        let index = build_lexical(&records);
        let extras = build_records(&records);
        let encoded: BTreeMap<String, Vec<u8>> = encode_lexical(&index, Some(&extras))
            .unwrap()
            .into_iter()
            .map(|file| (file.name, file.bytes))
            .collect();
        assert!(encoded.contains_key(META_FILE));
        assert!(encoded.contains_key(RECORDS_FILE));

        let tmp = tempfile::tempdir().unwrap();
        write_lexical(&index, Some(&extras), tmp.path()).unwrap();
        assert_eq!(read_dir_bytes(tmp.path()), encoded);
    }

    #[test]
    fn encode_lexical_omits_records_without_extras() {
        let records = records_with(|_| {});
        let files =
            encode_lexical(&build_lexical(&records), Some(&build_records(&records))).unwrap();
        assert!(files.iter().all(|file| file.name != RECORDS_FILE));
        assert_eq!(files.last().map(|file| file.name.as_str()), Some(META_FILE));
    }

    #[test]
    fn encode_lexical_names_the_records_file_on_mismatch() {
        let records = records_with(|r| r.boost = 1.2);
        let extras = build_records(&records[..1]);
        match encode_lexical(&build_lexical(&records), Some(&extras)) {
            Err(SearchError::Encode { path, .. }) => assert_eq!(path, Path::new(RECORDS_FILE)),
            other => panic!("expected an encode error, got {other:?}"),
        }
    }

    #[test]
    fn records_count_must_match_documents() {
        let tmp = tempfile::tempdir().unwrap();
        let records = records_with(|r| r.boost = 1.2);
        let extras = build_records(&records[..1]);
        match write_lexical(&build_lexical(&records), Some(&extras), tmp.path()) {
            Err(SearchError::Encode { path, .. }) => {
                assert_eq!(path, tmp.path().join(RECORDS_FILE))
            }
            other => panic!("expected an encode error, got {other:?}"),
        }
    }
}
