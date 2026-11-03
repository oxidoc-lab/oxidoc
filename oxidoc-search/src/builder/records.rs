//! Records side file built from input records.

use std::path::Path;

use super::write::write_file;
use crate::error::SearchError;
use crate::record::Record;
use crate::records::{DocRecord, FieldRange, RECORDS_FILE, SearchRecords, encode_records};

/// Builds the ranking data of `records`, in doc id order.
///
/// Field ranges index the body that [`Record::body`] returns, which is
/// `DocMetadata.text` in the lexical index.
pub fn build_records(records: &[Record]) -> SearchRecords {
    let docs = records
        .iter()
        .map(|record| DocRecord {
            tags: record.tags.clone(),
            boost: record.boost,
            fields: record
                .fields
                .iter()
                .zip(record.field_ranges())
                .map(|(field, range)| FieldRange {
                    name: field.name.clone(),
                    start: range.start,
                    end: range.end,
                    weight: field.weight,
                })
                .collect(),
        })
        .collect();
    SearchRecords::new(docs)
}

/// Writes `search-records.bin` into `dir` when `records` has extras.
///
/// Without extras it removes a `search-records.bin` left by an earlier build.
/// `doc_count` must equal the number of record entries.
pub(super) fn write_records(
    records: Option<&SearchRecords>,
    doc_count: usize,
    dir: &Path,
) -> Result<(), SearchError> {
    let path = dir.join(RECORDS_FILE);
    let Some(records) = records.filter(|r| r.has_extras()) else {
        return match std::fs::remove_file(&path) {
            Err(source) if source.kind() != std::io::ErrorKind::NotFound => {
                Err(SearchError::Write { path, source })
            }
            _ => Ok(()),
        };
    };
    if records.docs.len() != doc_count {
        return Err(SearchError::Encode {
            path,
            message: format!(
                "{} record entries for {doc_count} documents; build both from the same records",
                records.docs.len()
            ),
        });
    }
    let encoded = encode_records(records).map_err(|e| SearchError::Encode {
        path: path.clone(),
        message: e.to_string(),
    })?;
    write_file(path, &encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Field;

    #[test]
    fn build_records_maps_fields_to_body_ranges() {
        let mut record = Record::new("oxidoc", "/oxidoc", "oxidoc");
        record.fields.push(Field {
            weight: 4.0,
            ..Field::new("name", "oxidoc")
        });
        record.fields.push(Field::new("readme", "Docs engine"));
        record.tags.push(("kind".to_string(), "app".to_string()));
        record.boost = 1.3;

        let built = build_records(std::slice::from_ref(&record));
        let doc = &built.docs[0];
        let body = record.body();
        let spans: Vec<(&str, &str, f32)> = doc
            .fields
            .iter()
            .map(|f| (f.name.as_str(), &body[f.start..f.end], f.weight))
            .collect();
        assert_eq!(
            spans,
            vec![("name", "oxidoc", 4.0), ("readme", "Docs engine", 1.0)]
        );
        assert_eq!(doc.tags, record.tags);
        assert_eq!(doc.boost, 1.3);
        assert!(built.has_extras());
    }
}
