//! Section ranking of candidate documents.

use std::collections::HashMap;
use std::ops::Range;

use super::matching::{context_snippet_at, find_all_match_offsets};
use super::resolve::resolve_tokens;
use super::scoring::{
    Segment, compute_phrase_boost, resolve_heading_breadcrumb, score_section, section_range,
};
use super::search::LexicalSearcher;
use crate::records::{DocRecord, Filter};
use crate::types::{DocMetadata, SearchResult as DocResult, SearchSource};

/// Title boost of the intro section when the page title covers the query.
const PAGE_TITLE_BOOST: f32 = 2.0;
/// Title boost of the other sections of that page.
const SUBSECTION_TITLE_BOOST: f32 = 1.2;

/// One ranked section hit with the doc id it came from.
pub(crate) struct Hit {
    /// Document of the hit.
    pub doc_id: u32,
    /// The hit as returned to callers.
    pub result: DocResult,
}

/// Sorts hits by score, highest first. Ties keep their order.
pub(crate) fn sort_hits(hits: &mut [Hit]) {
    hits.sort_by(|a, b| b.result.score.total_cmp(&a.result.score));
}

/// Returns true when every query token covers a word of `title`.
///
/// A doc whose title covers all query tokens is likely the canonical page for the
/// query. A token covers a title word if it's stem-equal OR a prefix of the title
/// word (so "ve" still matches a title containing "Versioning").
fn title_covers_tokens(title: &str, tokens: &[String], raw_tokens: &[String]) -> bool {
    let title_words: Vec<(String, String)> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let lower = w.to_lowercase();
            let stem = oxidoc_text::stem(&lower);
            (lower, stem)
        })
        .collect();
    if title_words.is_empty() {
        return false;
    }
    tokens.iter().enumerate().all(|(i, t)| {
        let raw = raw_tokens.get(i).map(|s| s.as_str()).unwrap_or(t.as_str());
        title_words.iter().any(|(lower, stem)| {
            stem == t || lower.starts_with(t.as_str()) || lower.starts_with(raw)
        })
    })
}

/// Splits the section `range` of `text` into weighted field segments.
///
/// Without a record, or when no field overlaps the section, the whole section
/// is one segment of weight 1.0.
fn section_segments<'a>(
    text: &'a str,
    range: Range<usize>,
    record: Option<&DocRecord>,
) -> Vec<Segment<'a>> {
    let fields: Vec<Segment<'a>> = record
        .map(|r| {
            r.fields
                .iter()
                .filter_map(|f| {
                    let start = f.start.max(range.start);
                    let end = f.end.min(range.end);
                    if start >= end {
                        return None;
                    }
                    let text = text.get(start..end)?;
                    Some(Segment {
                        text,
                        weight: f.weight,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    if fields.is_empty() {
        vec![Segment {
            text: text.get(range).unwrap_or(""),
            weight: 1.0,
        }]
    } else {
        fields
    }
}

/// Returns the title boost of one section.
///
/// With a record that has a `name` field, the name weight replaces the page-title boost.
fn title_boost(title_match: bool, intro: bool, record: Option<&DocRecord>) -> f32 {
    if !title_match {
        return 1.0;
    }
    match record.and_then(DocRecord::name_weight) {
        Some(weight) => weight,
        None if intro => PAGE_TITLE_BOOST,
        None => SUBSECTION_TITLE_BOOST,
    }
}

impl LexicalSearcher {
    /// Scores every matching section of the documents that pass `filters`.
    ///
    /// Returns all hits sorted by score, without boost and without truncation.
    /// Candidates are visited in doc id order, so equal scores keep doc id order.
    pub(crate) fn search_hits(&self, text: &str, filters: &[Filter]) -> Vec<Hit> {
        let text = text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        let pairs = oxidoc_text::tokenize_query(text);
        if pairs.is_empty() {
            return Vec::new();
        }
        let tokens: Vec<String> = pairs.iter().map(|(s, _)| s.clone()).collect();
        let raw_tokens: Vec<String> = pairs.iter().map(|(_, r)| r.clone()).collect();

        let allow = |doc_id: u32| self.passes_filters(doc_id, filters);
        let resolved = resolve_tokens(&tokens, &raw_tokens, &self.postings, &allow);
        let num_tokens = tokens.len();
        let mut hits: Vec<Hit> = Vec::new();

        let doc_map: HashMap<u32, &DocMetadata> =
            self.documents.iter().map(|d| (d.id, d)).collect();
        let mut candidates: Vec<u32> = resolved.candidate_docs.iter().copied().collect();
        candidates.sort_unstable();

        for doc_id in candidates {
            let doc = match doc_map.get(&doc_id) {
                Some(d) => d,
                None => continue,
            };
            let record = self.record(doc_id);

            let mut terms = resolved
                .doc_matched_keys
                .get(&doc_id)
                .cloned()
                .unwrap_or_default();
            terms.sort();
            terms.dedup();

            let mut highlight_terms = resolved
                .doc_fuzzy_keys
                .get(&doc_id)
                .cloned()
                .unwrap_or_default();
            highlight_terms.sort();
            highlight_terms.dedup();

            // Compute OR penalty if AND failed
            let and_penalty = if !resolved.use_and {
                let matched_tokens = resolved
                    .per_token_doc_ids
                    .iter()
                    .filter(|ids| ids.contains(&doc_id))
                    .count();
                matched_tokens as f32 / num_tokens as f32
            } else {
                1.0
            };

            // Compute phrase boost
            let phrase_boost =
                compute_phrase_boost(&tokens, &resolved.token_postings_for_phrase, doc_id);

            if doc.text.is_empty() {
                hits.push(Hit {
                    doc_id,
                    result: DocResult {
                        title: doc.title.clone(),
                        path: doc.path.clone(),
                        snippet: doc.snippet.clone(),
                        score: 0.1 * and_penalty * phrase_boost,
                        source: SearchSource::Lexical,
                        breadcrumb: vec![],
                        anchor: String::new(),
                        highlight_terms,
                    },
                });
                continue;
            }

            // Find all match offsets, group by section
            let offsets = find_all_match_offsets(&doc.text, &terms);
            if offsets.is_empty() {
                continue;
            }

            let title_match = title_covers_tokens(&doc.title, &tokens, &raw_tokens);
            let mut seen_anchors: Vec<String> = Vec::new();
            for offset in &offsets {
                let (breadcrumb, anchor) =
                    resolve_heading_breadcrumb(&doc.title, &doc.headings, Some(*offset));
                if seen_anchors.contains(&anchor) {
                    continue;
                }
                seen_anchors.push(anchor.clone());

                // Score this section directly, with heading boost
                let range = section_range(&doc.text, &doc.headings, *offset);
                let segments = section_segments(&doc.text, range, record);
                let heading_title = if breadcrumb.len() > 1 {
                    breadcrumb.last().map(|s| s.as_str()).unwrap_or("")
                } else {
                    &doc.title
                };
                // Page-title match: boost the intro section (no subheading) strongly
                // so the dedicated page wins over subsections that share the term.
                let section_title_boost = title_boost(title_match, breadcrumb.is_empty(), record);
                let score =
                    score_section(&segments, heading_title, &tokens, &raw_tokens, num_tokens)
                        * and_penalty
                        * phrase_boost
                        * section_title_boost;

                if score <= 0.0 {
                    continue;
                }

                let snippet = context_snippet_at(&doc.text, *offset, &terms, 160);
                let path = if anchor.is_empty() {
                    doc.path.clone()
                } else {
                    format!("{}#{}", doc.path, anchor)
                };

                hits.push(Hit {
                    doc_id,
                    result: DocResult {
                        title: doc.title.clone(),
                        path,
                        snippet,
                        score,
                        source: SearchSource::Lexical,
                        breadcrumb,
                        anchor,
                        highlight_terms: highlight_terms.clone(),
                    },
                });
            }
        }

        sort_hits(&mut hits);
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::FieldRange;

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
            let got: Vec<(&str, f32)> = section_segments(text, range.clone(), rec)
                .iter()
                .map(|s| (s.text, s.weight))
                .collect();
            assert_eq!(got, want, "range {range:?}");
        }
    }

    #[test]
    fn title_boost_table() {
        let named = record(&[("name", 0, 1, 4.0)]);
        let unnamed = record(&[("body", 0, 1, 1.0)]);
        let cases = [
            (false, true, None, 1.0),
            (true, true, None, PAGE_TITLE_BOOST),
            (true, false, None, SUBSECTION_TITLE_BOOST),
            (true, true, Some(&unnamed), PAGE_TITLE_BOOST),
            (true, false, Some(&named), 4.0),
            (false, true, Some(&named), 1.0),
        ];
        for (title_match, intro, rec, want) in cases {
            assert_eq!(title_boost(title_match, intro, rec), want);
        }
    }

    #[test]
    fn title_covers_tokens_table() {
        let tokens = |q: &str| -> (Vec<String>, Vec<String>) {
            oxidoc_text::tokenize_query(q).into_iter().unzip()
        };
        let cases = [
            ("Versioning", "ve", true),
            ("Code Blocks", "code block", true),
            ("Code Blocks", "code fence", false),
            ("", "code", false),
        ];
        for (title, query, want) in cases {
            let (stems, raws) = tokens(query);
            assert_eq!(
                title_covers_tokens(title, &stems, &raws),
                want,
                "{title} / {query}"
            );
        }
    }
}
