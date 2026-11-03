//! Encoding, decoding and checking of `search-records.bin`.

use super::types::{RECORDS_FILE, RECORDS_VERSION, SearchRecords};
use crate::error::{SearchError, SearchResult};
use crate::types::DocMetadata;

/// Encodes `records` as rkyv bytes.
pub fn encode_records(records: &SearchRecords) -> SearchResult<Vec<u8>> {
    rkyv::to_bytes::<rkyv::rancor::Error>(records)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| SearchError::Serialization(format!("Failed to encode {RECORDS_FILE}: {e}")))
}

/// Decodes `search-records.bin` bytes.
///
/// Returns [`SearchError::UnsupportedVersion`] when the version is not [`RECORDS_VERSION`].
pub fn decode_records(data: &[u8]) -> SearchResult<SearchRecords> {
    let records = rkyv::from_bytes::<SearchRecords, rkyv::rancor::Error>(data)
        .map_err(|e| SearchError::IndexLoad(format!("Failed to decode {RECORDS_FILE}: {e}")))?;
    if records.version != RECORDS_VERSION {
        return Err(SearchError::UnsupportedVersion {
            file: RECORDS_FILE.to_string(),
            found: records.version,
            supported: RECORDS_VERSION,
        });
    }
    Ok(records)
}

/// Checks that `records` lines up with `documents`.
///
/// Each document needs one entry. Each field range must lie inside the document
/// text on char boundaries. Each boost and weight must be finite.
pub(crate) fn check_records(
    records: &SearchRecords,
    documents: &[DocMetadata],
) -> SearchResult<()> {
    if records.docs.len() != documents.len() {
        return Err(SearchError::IndexLoad(format!(
            "{RECORDS_FILE} has {} entries but the index has {} documents; rebuild both together",
            records.docs.len(),
            documents.len()
        )));
    }
    for (id, (record, doc)) in records.docs.iter().zip(documents).enumerate() {
        if !record.boost.is_finite() {
            return Err(bad_entry(
                id,
                format!("boost {} is not finite", record.boost),
            ));
        }
        for field in &record.fields {
            let in_text = field.start <= field.end
                && field.end <= doc.text.len()
                && doc.text.is_char_boundary(field.start)
                && doc.text.is_char_boundary(field.end);
            if !in_text {
                return Err(bad_entry(
                    id,
                    format!(
                        "field `{}` range {}..{} is outside the {}-byte text or splits a char",
                        field.name,
                        field.start,
                        field.end,
                        doc.text.len()
                    ),
                ));
            }
            if !field.weight.is_finite() {
                return Err(bad_entry(
                    id,
                    format!(
                        "field `{}` weight {} is not finite",
                        field.name, field.weight
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn bad_entry(id: usize, detail: String) -> SearchError {
    SearchError::IndexLoad(format!("{RECORDS_FILE} entry for doc {id}: {detail}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::types::{DocRecord, FieldRange};

    fn doc(text: &str) -> DocMetadata {
        DocMetadata {
            id: 0,
            title: "T".to_string(),
            path: "/t".to_string(),
            snippet: String::new(),
            text: text.to_string(),
            headings: vec![],
        }
    }

    fn record(start: usize, end: usize, boost: f32) -> DocRecord {
        DocRecord {
            tags: vec![("kind".to_string(), "library".to_string())],
            boost,
            fields: vec![FieldRange {
                name: "name".to_string(),
                start,
                end,
                weight: 2.0,
            }],
        }
    }

    #[test]
    fn records_round_trip() {
        let records = SearchRecords::new(vec![record(0, 4, 1.2), record(1, 3, 1.0)]);
        let bytes = encode_records(&records).unwrap();
        assert_eq!(decode_records(&bytes).unwrap(), records);
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut records = SearchRecords::new(vec![record(0, 4, 1.0)]);
        records.version = RECORDS_VERSION + 1;
        let bytes = encode_records(&records).unwrap();
        match decode_records(&bytes) {
            Err(SearchError::UnsupportedVersion {
                file,
                found,
                supported,
            }) => {
                assert_eq!(file, RECORDS_FILE);
                assert_eq!(found, RECORDS_VERSION + 1);
                assert_eq!(supported, RECORDS_VERSION);
            }
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(matches!(
            decode_records(&[1, 2, 3]),
            Err(SearchError::IndexLoad(_))
        ));
    }

    #[test]
    fn check_records_table() {
        let docs = vec![doc("abéd")];
        let cases = [
            (SearchRecords::new(vec![record(0, 5, 1.0)]), true),
            (SearchRecords::new(vec![record(0, 2, 1.0)]), true),
            (SearchRecords::new(vec![]), false),
            (
                SearchRecords::new(vec![record(0, 1, 1.0), record(0, 1, 1.0)]),
                false,
            ),
            (SearchRecords::new(vec![record(0, 6, 1.0)]), false),
            (SearchRecords::new(vec![record(3, 2, 1.0)]), false),
            (SearchRecords::new(vec![record(0, 3, 1.0)]), false),
            (SearchRecords::new(vec![record(0, 2, f32::NAN)]), false),
        ];
        for (records, ok) in cases {
            assert_eq!(check_records(&records, &docs).is_ok(), ok, "{records:?}");
        }
    }
}
