//! Section ranking of candidate documents.

use super::matching::context_snippet_at;
use super::query_scorer::QueryScorer;
use super::resolve::resolve_tokens;
use super::scoring::{PhraseIndex, compute_phrase_boost, heading_breadcrumb, score_section};
use super::search::LexicalSearcher;
use super::section_hits::first_match_per_section;
use crate::records::{DocRecord, Filter};
use crate::types::{SearchResult as DocResult, SearchSource};

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
fn title_covers_tokens(title_words: &[usize], scorer: &QueryScorer<'_>) -> bool {
    if title_words.is_empty() {
        return false;
    }
    (0..scorer.token_count()).all(|t| scorer.covers(t, title_words))
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

        let (tokens, raw_tokens): (Vec<String>, Vec<String>) =
            oxidoc_text::tokenize_query(text).into_iter().unzip();
        if tokens.is_empty() {
            return Vec::new();
        }

        let allow = |doc_id: u32| self.passes_filters(doc_id, filters);
        let resolved = resolve_tokens(&tokens, &raw_tokens, &self.postings, &allow);
        let num_tokens = tokens.len();
        let mut hits: Vec<Hit> = Vec::new();

        let phrase = PhraseIndex::new(&resolved.token_postings_for_phrase);
        let matches = self.match_index();
        let mut scorer = QueryScorer::new(&matches.vocab, &tokens, &raw_tokens);
        let mut candidates: Vec<u32> = resolved.candidate_docs.iter().copied().collect();
        candidates.sort_unstable();

        for doc_id in candidates {
            let Some((pos, doc)) = self.doc_at(doc_id) else {
                continue;
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
            let phrase_boost = compute_phrase_boost(&tokens, &phrase, doc_id);
            let highlight_terms: Vec<String> =
                highlight_terms.into_iter().map(str::to_string).collect();

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
            let Some(words) = matches.doc(pos) else {
                continue;
            };
            let lower_text = words.lower_text(doc);
            // First match of each section that holds a match, in text order.
            let firsts = first_match_per_section(lower_text, &terms, &doc.headings);
            if firsts.is_empty() {
                continue;
            }

            let title_match = title_covers_tokens(&words.title, &scorer);
            let mut seen_anchors: Vec<&str> = Vec::new();
            for &(offset, heading) in &firsts {
                let anchor = heading
                    .and_then(|i| doc.headings.get(i))
                    .map_or("", |h| h.anchor.as_str());
                if seen_anchors.contains(&anchor) {
                    continue;
                }
                seen_anchors.push(anchor);

                // Score this section directly, with heading boost
                let Some(section) = words.section(heading) else {
                    continue;
                };
                let breadcrumb = heading_breadcrumb(&doc.title, &doc.headings, heading);
                // Page-title match: boost the intro section (no subheading) strongly
                // so the dedicated page wins over subsections that share the term.
                let section_title_boost = title_boost(title_match, breadcrumb.is_empty(), record);
                let score = score_section(section, &mut scorer, num_tokens)
                    * and_penalty
                    * phrase_boost
                    * section_title_boost;

                if score <= 0.0 {
                    continue;
                }

                let snippet = context_snippet_at(&doc.text, lower_text, offset, &terms, 160);
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
                        anchor: anchor.to_string(),
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
    use crate::lexical::match_index::MatchIndex;
    use crate::records::FieldRange;
    use crate::types::DocMetadata;

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
            let doc = DocMetadata {
                id: 0,
                title: title.to_string(),
                path: String::new(),
                snippet: String::new(),
                text: String::new(),
                headings: vec![],
            };
            let index = MatchIndex::new(std::slice::from_ref(&doc), None);
            let title_words = index.doc(0).map_or(&[][..], |d| d.title.as_slice());
            let scorer = QueryScorer::new(&index.vocab, &stems, &raws);
            assert_eq!(
                title_covers_tokens(title_words, &scorer),
                want,
                "{title} / {query}"
            );
        }
    }
}
