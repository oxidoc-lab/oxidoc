//! Words and lowercased text of every document, built on the first query.

use std::ops::Range;

use super::scoring::section_range_at;
use super::vocab::{Vocab, VocabBuilder, split_words};
use crate::records::DocRecord;
use crate::types::DocMetadata;

/// Match data of all documents over one shared vocabulary.
pub(super) struct MatchIndex {
    /// Words and stems that the word ids below refer to.
    pub vocab: Vocab,
    /// Match data per position in the searcher's document list.
    docs: Vec<DocWords>,
}

/// Lowercased text, title words and section words of one document.
pub(super) struct DocWords {
    /// `DocMetadata.text` lowercased, or `None` when lowercasing changes nothing.
    lower_text: Option<String>,
    /// Distinct lowercased title word ids, for the page-title check.
    pub title: Vec<usize>,
    /// Section words: slot 0 is the intro, slot `i + 1` is the section of heading `i`.
    sections: Vec<SectionWords>,
}

/// Word ids of one section, per weighted field segment, and of its heading.
pub(super) struct SectionWords {
    /// Distinct word ids of each segment with the segment weight.
    pub segments: Vec<(Vec<usize>, f32)>,
    /// Distinct word ids of the section heading, with camelCase parts.
    pub heading: Vec<usize>,
}

impl MatchIndex {
    /// Builds the match data of `documents`.
    ///
    /// `records` are indexed by doc id. Without records each section is one segment of
    /// weight 1.0.
    pub(super) fn new(documents: &[DocMetadata], records: Option<&[DocRecord]>) -> Self {
        let mut builder = VocabBuilder::default();
        let docs = documents
            .iter()
            .map(|doc| {
                let record = records.and_then(|r| r.get(doc.id as usize));
                DocWords::new(doc, record, &mut builder)
            })
            .collect();
        Self {
            vocab: builder.finish(),
            docs,
        }
    }

    /// Returns the match data of the document at `pos` in the searcher's list.
    pub(super) fn doc(&self, pos: usize) -> Option<&DocWords> {
        self.docs.get(pos)
    }
}

impl DocWords {
    /// Builds the match data of `doc`, with field weights from `record`.
    fn new(doc: &DocMetadata, record: Option<&DocRecord>, builder: &mut VocabBuilder) -> Self {
        let lower_titles: Vec<String> = doc
            .title
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect();
        let title = builder.distinct_ids(lower_titles.iter().map(String::as_str));
        let sections = std::iter::once(None)
            .chain((0..doc.headings.len()).map(Some))
            .map(|heading| SectionWords::new(doc, record, heading, builder))
            .collect();
        let lower = doc.text.to_lowercase();
        Self {
            lower_text: (lower != doc.text).then_some(lower),
            title,
            sections,
        }
    }

    /// Returns the lowercased text of `doc`, the document this data was built from.
    pub(super) fn lower_text<'a>(&'a self, doc: &'a DocMetadata) -> &'a str {
        self.lower_text.as_deref().unwrap_or(&doc.text)
    }

    /// Returns the words of the section under heading `heading`, or of the intro for `None`.
    ///
    /// Returns `None` when `heading` is not a heading index of the document.
    pub(super) fn section(&self, heading: Option<usize>) -> Option<&SectionWords> {
        self.sections.get(heading.map_or(0, |i| i + 1))
    }
}

impl SectionWords {
    /// Splits the section of `heading` into weighted segments and collects their words.
    fn new(
        doc: &DocMetadata,
        record: Option<&DocRecord>,
        heading: Option<usize>,
        builder: &mut VocabBuilder,
    ) -> Self {
        let range = section_range_at(doc.text.len(), &doc.headings, heading);
        let segments = section_segments(&doc.text, range, record)
            .into_iter()
            .map(|(text, weight)| {
                let lower = text.to_lowercase();
                (builder.distinct_ids(split_words(&lower)), weight)
            })
            .collect();
        let title = heading
            .and_then(|i| doc.headings.get(i))
            .map_or(doc.title.as_str(), |h| h.title.as_str());
        Self {
            segments,
            heading: builder.distinct_ids(heading_words(title).iter().map(String::as_str)),
        }
    }
}

/// Returns the lowercased words of `title` and the camelCase parts of each word.
///
/// A part shorter than two bytes is dropped.
fn heading_words(title: &str) -> Vec<String> {
    let lower = title.to_lowercase();
    let mut expanded: Vec<String> = Vec::new();
    for word in split_words(&lower) {
        expanded.push(word.to_string());
        let parts = oxidoc_text::split_camel_case(word);
        if parts.len() > 1 {
            for part in parts {
                let part = part.to_lowercase();
                if part.len() > 1 {
                    expanded.push(part);
                }
            }
        }
    }
    expanded
}

/// Splits the section `range` of `text` into weighted field slices.
///
/// Without a record, or when no field overlaps the section, the whole section
/// is one slice of weight 1.0.
fn section_segments<'a>(
    text: &'a str,
    range: Range<usize>,
    record: Option<&DocRecord>,
) -> Vec<(&'a str, f32)> {
    let fields: Vec<(&'a str, f32)> = record
        .map(|r| {
            r.fields
                .iter()
                .filter_map(|f| {
                    let start = f.start.max(range.start);
                    let end = f.end.min(range.end);
                    if start >= end {
                        return None;
                    }
                    Some((text.get(start..end)?, f.weight))
                })
                .collect()
        })
        .unwrap_or_default();
    if fields.is_empty() {
        vec![(text.get(range).unwrap_or(""), 1.0)]
    } else {
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::FieldRange;
    use crate::types::HeadingPos;

    fn record(fields: &[(&str, usize, usize, f32)]) -> DocRecord {
        DocRecord {
            tags: vec![],
            boost: 1.0,
            fields: fields
                .iter()
                .map(|&(name, start, end, weight)| FieldRange {
                    name: name.to_string(),
                    start,
                    end,
                    weight,
                })
                .collect(),
        }
    }

    /// One case: a section range, the record, and the expected weighted slices.
    type SegmentCase<'a> = (Range<usize>, Option<&'a DocRecord>, Vec<(&'a str, f32)>);

    #[test]
    fn section_segments_table() {
        let text = "alpha\nbeta gamma\ndelta";
        let two = record(&[("name", 0, 5, 3.0), ("readme", 6, 22, 1.0)]);
        let cases: [SegmentCase<'_>; 4] = [
            (0..22, None, vec![(text, 1.0)]),
            (
                0..22,
                Some(&two),
                vec![("alpha", 3.0), ("beta gamma\ndelta", 1.0)],
            ),
            (11..22, Some(&two), vec![("gamma\ndelta", 1.0)]),
            (5..6, Some(&two), vec![("\n", 1.0)]),
        ];
        for (range, rec, want) in cases {
            assert_eq!(
                section_segments(text, range.clone(), rec),
                want,
                "range {range:?}"
            );
        }
    }

    #[test]
    fn section_words_follow_headings() {
        let doc = DocMetadata {
            id: 0,
            title: "Guide".to_string(),
            path: "/g".to_string(),
            snippet: String::new(),
            text: "Intro words setup steps".to_string(),
            headings: vec![HeadingPos {
                title: "Setup CodeBlock".to_string(),
                anchor: "setup".to_string(),
                depth: 2,
                offset: 12,
            }],
        };
        let index = MatchIndex::new(std::slice::from_ref(&doc), None);
        let words =
            |ids: &[usize]| -> Vec<&str> { ids.iter().map(|&id| index.vocab.get(id).0).collect() };
        let Some(words_of_doc) = index.doc(0) else {
            panic!("doc 0 exists");
        };
        assert!(index.doc(1).is_none());
        assert!(words_of_doc.section(Some(1)).is_none());
        let (Some(intro), Some(setup)) =
            (words_of_doc.section(None), words_of_doc.section(Some(0)))
        else {
            panic!("both sections exist");
        };
        assert_eq!(words(&intro.segments[0].0), ["intro", "words"]);
        assert_eq!(words(&intro.heading), ["guide"]);
        assert_eq!(words(&setup.segments[0].0), ["setup", "steps"]);
        assert_eq!(words(&setup.heading), ["setup", "codeblock"]);
        assert_eq!(words(&words_of_doc.title), ["guide"]);
        assert_eq!(words_of_doc.lower_text(&doc), "intro words setup steps");
    }
}
