//! Input records for the index builder.

use std::ops::Range;

use crate::types::HeadingPos;

/// One searchable document handed to the index builder.
///
/// The builder joins the field texts with `\n`, in order, to form the body.
/// Heading offsets index that body.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// Stable caller-side identifier. The builder assigns doc ids by input order.
    pub id: String,
    /// Link target stored as `DocMetadata.path`.
    pub url: String,
    /// Display title.
    pub title: String,
    /// Text fields that form the body, in order.
    pub fields: Vec<Field>,
    /// Heading positions as byte offsets into the body.
    pub headings: Vec<HeadingPos>,
    /// Name and value pairs for filtering.
    pub tags: Vec<(String, String)>,
    /// Ranking multiplier for the whole record. Defaults to 1.0.
    pub boost: f32,
    /// Snippet override. `None` derives the snippet from the body.
    pub snippet: Option<String>,
}

/// One named text field of a [`Record`].
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Field name, such as `body`.
    pub name: String,
    /// Plain text of the field.
    pub text: String,
    /// Ranking multiplier for terms in this field. Defaults to 1.0.
    pub weight: f32,
}

impl Field {
    /// Creates a field with weight 1.0.
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
            weight: 1.0,
        }
    }
}

impl Record {
    /// Creates a record with no fields, headings or tags, boost 1.0 and no snippet override.
    pub fn new(id: impl Into<String>, url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            title: title.into(),
            fields: Vec::new(),
            headings: Vec::new(),
            tags: Vec::new(),
            boost: 1.0,
            snippet: None,
        }
    }

    /// Returns the body: field texts joined with `\n`, in order.
    pub fn body(&self) -> String {
        let texts: Vec<&str> = self.fields.iter().map(|f| f.text.as_str()).collect();
        texts.join("\n")
    }

    /// Returns the byte range of each field in [`Record::body`], in field order.
    pub fn field_ranges(&self) -> Vec<Range<usize>> {
        let mut ranges = Vec::with_capacity(self.fields.len());
        let mut start = 0;
        for field in &self.fields {
            let end = start + field.text.len();
            ranges.push(start..end);
            start = end + 1;
        }
        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_record_has_defaults() {
        let record = Record::new("a", "/a", "A");
        assert_eq!(record.boost, 1.0);
        assert!(record.fields.is_empty());
        assert!(record.tags.is_empty());
        assert!(record.snippet.is_none());
        assert_eq!(record.body(), "");
    }

    #[test]
    fn new_field_has_unit_weight() {
        assert_eq!(Field::new("body", "text").weight, 1.0);
    }

    #[test]
    fn body_joins_fields_with_newline() {
        let mut record = Record::new("a", "/a", "A");
        record.fields.push(Field::new("summary", "first"));
        record.fields.push(Field::new("body", "second part"));
        assert_eq!(record.body(), "first\nsecond part");
    }

    #[test]
    fn field_ranges_index_the_body() {
        let mut record = Record::new("a", "/a", "A");
        record.fields.push(Field::new("summary", "first"));
        record.fields.push(Field::new("empty", ""));
        record.fields.push(Field::new("body", "second part"));
        let body = record.body();
        let ranges = record.field_ranges();
        assert_eq!(ranges, vec![0..5, 6..6, 7..18]);
        for (field, range) in record.fields.iter().zip(ranges) {
            assert_eq!(&body[range], field.text);
        }
    }
}
