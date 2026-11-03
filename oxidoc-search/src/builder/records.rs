//! Records side file built from input records.

use std::path::PathBuf;

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

/// Encodes `search-records.bin` when `records` has extras. `None` without extras.
///
/// `doc_count` must equal the number of record entries. An error names the
/// file without a directory.
pub(super) fn encode_records_file(
    records: Option<&SearchRecords>,
    doc_count: usize,
) -> Result<Option<Vec<u8>>, SearchError> {
    let Some(records) = records.filter(|r| r.has_extras()) else {
        return Ok(None);
    };
    if records.docs.len() != doc_count {
        return Err(SearchError::Encode {
            path: PathBuf::from(RECORDS_FILE),
            message: format!(
                "{} record entries for {doc_count} documents; build both from the same records",
                records.docs.len()
            ),
        });
    }
    encode_records(records)
        .map(Some)
        .map_err(|e| SearchError::Encode {
            path: PathBuf::from(RECORDS_FILE),
            message: e.to_string(),
        })
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
