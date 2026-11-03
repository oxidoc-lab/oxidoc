//! Per-document ranking data stored in `search-records.bin`.

/// File name of the records side file.
pub const RECORDS_FILE: &str = "search-records.bin";

/// Format version this crate writes and reads.
pub const RECORDS_VERSION: u32 = 1;

/// Field name whose weight replaces the page-title boost.
pub const NAME_FIELD: &str = "name";

/// Contents of `search-records.bin`: one entry per document, in doc id order.
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(crate = rkyv)]
pub struct SearchRecords {
    /// Format version. Readers reject any value other than [`RECORDS_VERSION`].
    pub version: u32,
    /// Ranking data per document. Entry `i` belongs to doc id `i`.
    pub docs: Vec<DocRecord>,
}

/// Ranking data of one document.
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(crate = rkyv)]
pub struct DocRecord {
    /// Name and value pairs for filtering and facets.
    pub tags: Vec<(String, String)>,
    /// Score multiplier for the whole document. The runtime clamps it.
    pub boost: f32,
    /// Fields of the document body, in body order.
    pub fields: Vec<FieldRange>,
}

/// One named field as a byte range of `DocMetadata.text`.
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(crate = rkyv)]
pub struct FieldRange {
    /// Field name, such as `name` or `readme`.
    pub name: String,
    /// First byte of the field in the document text.
    pub start: usize,
    /// Byte after the last byte of the field.
    pub end: usize,
    /// Multiplier for matches inside the field.
    pub weight: f32,
}

impl SearchRecords {
    /// Creates a records file of the current version.
    pub fn new(docs: Vec<DocRecord>) -> Self {
        Self {
            version: RECORDS_VERSION,
            docs,
        }
    }

    /// Returns true when any document differs from a plain page.
    ///
    /// A plain page has no tags, boost 1.0, and at most one field of weight 1.0.
    pub fn has_extras(&self) -> bool {
        self.docs.iter().any(DocRecord::has_extras)
    }
}

impl DocRecord {
    /// Returns true when the document has tags, a boost other than 1.0,
    /// more than one field, or a field weight other than 1.0.
    pub fn has_extras(&self) -> bool {
        !self.tags.is_empty()
            || self.boost != 1.0
            || self.fields.len() > 1
            || self.fields.iter().any(|f| f.weight != 1.0)
    }

    /// Returns the weight of the [`NAME_FIELD`] field, if the document has one.
    pub fn name_weight(&self) -> Option<f32> {
        self.fields
            .iter()
            .find(|f| f.name == NAME_FIELD)
            .map(|f| f.weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, weight: f32) -> FieldRange {
        FieldRange {
            name: name.to_string(),
            start: 0,
            end: 0,
            weight,
        }
    }

    fn plain() -> DocRecord {
        DocRecord {
            tags: vec![],
            boost: 1.0,
            fields: vec![field("body", 1.0)],
        }
    }

    #[test]
    fn has_extras_table() {
        let tagged = DocRecord {
            tags: vec![("kind".to_string(), "library".to_string())],
            ..plain()
        };
        let boosted = DocRecord {
            boost: 1.2,
            ..plain()
        };
        let two_fields = DocRecord {
            fields: vec![field("name", 1.0), field("body", 1.0)],
            ..plain()
        };
        let weighted = DocRecord {
            fields: vec![field("body", 2.0)],
            ..plain()
        };
        let empty = DocRecord {
            fields: vec![],
            ..plain()
        };
        let cases = [
            (plain(), false),
            (empty, false),
            (tagged, true),
            (boosted, true),
            (two_fields, true),
            (weighted, true),
        ];
        for (doc, want) in cases {
            assert_eq!(doc.has_extras(), want, "{doc:?}");
            assert_eq!(SearchRecords::new(vec![plain(), doc]).has_extras(), want);
        }
    }

    #[test]
    fn name_weight_finds_name_field() {
        let doc = DocRecord {
            fields: vec![field("name", 4.0), field("readme", 1.0)],
            ..plain()
        };
        assert_eq!(doc.name_weight(), Some(4.0));
        assert_eq!(plain().name_weight(), None);
    }
}
