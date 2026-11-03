//! Document metadata built from records.

use crate::record::Record;
use crate::types::DocMetadata;

/// Maximum snippet length in bytes, before the ellipsis.
pub const SNIPPET_LEN: usize = 160;

/// Returns the trimmed text, cut to at most `max_len` bytes with a trailing `…`.
///
/// A cut inside a multi-byte char moves back to the previous char boundary.
pub fn create_snippet(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        let mut end = max_len;
        while !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &trimmed[..end])
    }
}

/// Builds the metadata for the record at input position `id`.
pub(crate) fn doc_metadata(id: u32, record: &Record, body: String) -> DocMetadata {
    let snippet = match &record.snippet {
        Some(snippet) => snippet.clone(),
        None => create_snippet(&body, SNIPPET_LEN),
    };
    DocMetadata {
        id,
        title: record.title.clone(),
        path: record.url.clone(),
        snippet,
        text: body,
        headings: record.headings.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Field;

    #[test]
    fn create_snippet_truncates() {
        let text = "This is a very long piece of text that should be truncated";
        let snippet = create_snippet(text, 30);
        assert!(snippet.len() <= 33);
        assert!(snippet.ends_with('…'));
    }

    #[test]
    fn create_snippet_short_text() {
        assert_eq!(create_snippet("Short", 30), "Short");
    }

    #[test]
    fn create_snippet_backs_off_to_char_boundary() {
        // "é" is two bytes, so byte 3 falls inside it.
        assert_eq!(create_snippet("abéd", 3), "ab…");
    }

    #[test]
    fn doc_metadata_uses_url_and_body() {
        let mut record = Record::new("guide", "/guide", "Guide");
        record.fields.push(Field::new("body", "  Guide text  "));
        let body = record.body();
        let meta = doc_metadata(4, &record, body);
        assert_eq!(meta.id, 4);
        assert_eq!(meta.path, "/guide");
        assert_eq!(meta.title, "Guide");
        assert_eq!(meta.snippet, "Guide text");
        assert_eq!(meta.text, "  Guide text  ");
    }

    #[test]
    fn doc_metadata_keeps_snippet_override() {
        let mut record = Record::new("a", "/a", "A");
        record.fields.push(Field::new("body", "body text"));
        record.snippet = Some("custom".to_string());
        let body = record.body();
        assert_eq!(doc_metadata(0, &record, body).snippet, "custom");
    }
}
