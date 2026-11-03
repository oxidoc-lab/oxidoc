use std::collections::HashMap;
use std::ops::Range;

use super::match_index::SectionWords;
use super::query_scorer::QueryScorer;
use crate::types::{HeadingPos, Posting};

/// Scores a section by how its words match the query tokens.
///
/// Each token keeps its best weighted word score over the section segments.
/// Multi-token: all tokens matched → 3x bonus, else scaled by the matched share.
/// Heading matches then multiply the total.
pub(super) fn score_section(
    section: &SectionWords,
    scorer: &mut QueryScorer<'_>,
    num_tokens: usize,
) -> f32 {
    let mut total = 0.0_f32;
    let mut matched_count = 0usize;

    for t in 0..scorer.token_count() {
        let mut best = 0.0_f32;
        for (words, weight) in &section.segments {
            best = best.max(scorer.best_body(t, words) * weight);
        }
        if best > 0.0 {
            matched_count += 1;
        }
        total += best;
    }

    if num_tokens > 1 && matched_count == num_tokens {
        total *= 3.0;
    } else if num_tokens > 1 {
        total *= matched_count as f32 / num_tokens as f32;
    }

    // Heading boost: score tokens against heading words using same position+coverage logic.
    // The heading words include camelCase parts (e.g. "CodeBlock" → ["code","block"]).
    if !section.heading.is_empty() {
        let mut heading_score = 0.0_f32;
        let mut heading_hits = 0usize;
        for t in 0..scorer.token_count() {
            let best = scorer.best_heading(t, &section.heading);
            if best > 0.0 {
                heading_hits += 1;
                heading_score += best;
            }
        }
        if heading_hits > 0 {
            let heading_ratio = heading_hits as f32 / num_tokens.max(1) as f32;
            total *= 1.0 + heading_ratio * heading_score * 2.0;
        }
    }

    total
}

/// Token positions of matched postings, per query token, grouped by document.
pub(super) struct PhraseIndex<'a> {
    /// For each query token: doc id → position lists of every matched key.
    per_token: Vec<HashMap<u32, Vec<&'a [u32]>>>,
}

impl<'a> PhraseIndex<'a> {
    /// Groups `token_postings` by document. Builds nothing for fewer than two tokens.
    pub(super) fn new(token_postings: &[Vec<(&'a str, &'a [Posting])>]) -> Self {
        if token_postings.len() < 2 {
            return Self {
                per_token: Vec::new(),
            };
        }
        let per_token = token_postings
            .iter()
            .map(|keys| {
                let mut by_doc: HashMap<u32, Vec<&'a [u32]>> = HashMap::new();
                for (_key, posts) in keys {
                    for post in posts.iter() {
                        by_doc
                            .entry(post.doc_id)
                            .or_default()
                            .push(post.positions.as_slice());
                    }
                }
                by_doc
            })
            .collect();
        Self { per_token }
    }

    /// Returns true when token `pair` and token `pair + 1` sit at adjacent positions in `doc_id`.
    fn sequential(&self, pair: usize, doc_id: u32) -> bool {
        let lists = |i: usize| {
            self.per_token
                .get(i)
                .and_then(|m| m.get(&doc_id))
                .map_or(&[][..], Vec::as_slice)
        };
        let (lists_a, lists_b) = (lists(pair), lists(pair + 1));
        lists_a.iter().any(|positions_a| {
            positions_a.iter().any(|&pos_a| {
                lists_b
                    .iter()
                    .any(|positions_b| positions_b.iter().any(|&b| pos_a.checked_add(1) == Some(b)))
            })
        })
    }
}

/// Compute phrase boost based on positional data.
///
/// For consecutive query token pairs, checks if any matched postings have
/// sequential positions (pos_b == pos_a + 1).
/// - All consecutive pairs sequential → 5x boost
/// - Partial → proportional boost
pub(super) fn compute_phrase_boost(
    query_tokens: &[String],
    phrase: &PhraseIndex<'_>,
    doc_id: u32,
) -> f32 {
    if query_tokens.len() < 2 {
        return 1.0;
    }

    let num_pairs = query_tokens.len() - 1;
    let sequential_pairs = (0..num_pairs)
        .filter(|&pair| phrase.sequential(pair, doc_id))
        .count();

    if sequential_pairs == 0 {
        return 1.0;
    }

    let ratio = sequential_pairs as f32 / num_pairs as f32;
    // Full phrase match = 5x, partial = proportional
    1.0 + ratio * 4.0
}

/// Returns the byte range of the section of `heading`, from the heading to the next one.
///
/// `None` is the intro: the text before the first heading.
pub(super) fn section_range_at(
    text_len: usize,
    headings: &[HeadingPos],
    heading: Option<usize>,
) -> Range<usize> {
    match heading.and_then(|i| headings.get(i).map(|h| (i, h))) {
        None => 0..headings.first().map_or(text_len, |h| h.offset),
        Some((i, h)) => {
            let end = headings.get(i + 1).map_or(text_len, |next| next.offset);
            h.offset..end
        }
    }
}

/// Builds the breadcrumb of the section of `heading`.
///
/// The breadcrumb is the page title, the ancestor headings (lower depth), then the
/// heading itself, e.g. ["Page Title", "h2"]. The intro (`None`) has no breadcrumb.
pub(super) fn heading_breadcrumb(
    page_title: &str,
    headings: &[HeadingPos],
    heading: Option<usize>,
) -> Vec<String> {
    let Some((closest_idx, closest)) = heading.and_then(|i| headings.get(i).map(|h| (i, h))) else {
        return vec![];
    };

    let mut crumbs = vec![page_title.to_string()];

    // Walk backwards to find ancestor headings
    let mut need_depth = closest.depth.saturating_sub(1);
    let mut ancestors = Vec::new();
    for h in headings[..closest_idx].iter().rev() {
        if h.depth <= need_depth {
            ancestors.push(h.title.clone());
            if h.depth <= 2 {
                break;
            }
            need_depth = h.depth - 1;
        }
    }
    ancestors.reverse();
    crumbs.extend(ancestors);
    crumbs.push(closest.title.clone());
    crumbs
}
