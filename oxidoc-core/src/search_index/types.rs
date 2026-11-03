//! Search index types. The index types live in `oxidoc-search`; this module adds the page input.

pub use oxidoc_search::types::*;
use oxidoc_search::{Field, Record};

/// Plain text content extracted from a page (one per page).
#[derive(Debug, Clone)]
pub struct PageContent {
    /// Page title: the first H1, else the nav title.
    pub title: String,
    /// Page slug without the leading `/`.
    pub slug: String,
    /// Searchable plain text. Code blocks are left out.
    pub text: String,
    /// Heading positions as byte offsets into `text`.
    pub headings: Vec<HeadingPos>,
}

/// Maps a page to a builder record: url `/{slug}`, one `body` field, no tags, boost 1.0.
impl From<&PageContent> for Record {
    fn from(page: &PageContent) -> Self {
        let mut record = Record::new(page.slug.clone(), format!("/{}", page.slug), &page.title);
        record.fields.push(Field::new("body", page.text.clone()));
        record.headings = page.headings.clone();
        record
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_maps_to_single_body_record() {
        let page = PageContent {
            title: "Guide".to_string(),
            slug: "docs/guide".to_string(),
            text: "Intro. Setup here.".to_string(),
            headings: vec![HeadingPos {
                title: "Setup".to_string(),
                anchor: "setup".to_string(),
                depth: 2,
                offset: 7,
            }],
        };

        let record = Record::from(&page);
        assert_eq!(record.id, "docs/guide");
        assert_eq!(record.url, "/docs/guide");
        assert_eq!(record.title, "Guide");
        assert_eq!(record.fields.len(), 1);
        assert_eq!(record.fields[0].name, "body");
        assert_eq!(record.fields[0].weight, 1.0);
        assert_eq!(record.body(), page.text);
        assert_eq!(record.headings[0].offset, 7);
        assert!(record.tags.is_empty());
        assert_eq!(record.boost, 1.0);
        assert!(record.snippet.is_none());
    }
}
