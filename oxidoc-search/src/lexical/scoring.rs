use super::matching::{levenshtein, max_edit_distance};
use crate::types::Posting;

/// Score a section based on how query tokens match within it.
/// For each token, find the best matching word. Score = 1.0 - (match_position / word_length).
/// Earlier match in word = higher score. Exact match = 1.0.
/// Multi-token: all tokens matched → 3x bonus.
pub(super) fn score_section(
    section_text: &str,
    heading_title: &str,
    tokens: &[String],
    raw_tokens: &[String],
    num_tokens: usize,
) -> f32 {
    // Score a single word against (stem, raw) forms of a token.
    //
    // A token that's a prefix of the word (pos=0) scores 0.9 regardless of
    // word length — a typed prefix like "ve" should match "versioning" as
    // strongly as it matches "vercel", so the ranking is decided by other
    // signals (title match, heading boost) rather than whichever word happens
    // to be shorter. Non-prefix substring matches fall back to the original
    // position×coverage formula to keep mid-word matches weaker.
    let word_match_score = |word: &str, tk: &str, raw: &str| -> f32 {
        let mut best = 0.0_f32;
        for form in [tk, raw] {
            if let Some(pos) = word.find(form) {
                let score = if pos == 0 {
                    0.9
                } else {
                    let wlen = word.len().max(1) as f32;
                    let ps = 1.0 - (pos as f32 / wlen);
                    let cs = form.len() as f32 / wlen;
                    ps * cs
                };
                best = best.max(score);
            }
            if tk == raw {
                break;
            }
        }
        best
    };
    let lower = section_text.to_lowercase();
    let heading_lower = heading_title.to_lowercase();
    let heading_words: Vec<&str> = heading_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mut total = 0.0_f32;
    let mut matched_count = 0usize;

    for (i, tk) in tokens.iter().enumerate() {
        let raw = raw_tokens.get(i).map(|s| s.as_str()).unwrap_or(tk.as_str());
        let mut best = 0.0_f32;
        let max_dist = max_edit_distance(tk.len());
        for word in &words {
            if oxidoc_text::stem(word) == *tk {
                best = 1.0;
                break;
            }
            let score = word_match_score(word, tk, raw);
            if score > 0.0 {
                best = best.max(score);
                if score >= 1.0 {
                    break;
                }
            } else if max_dist > 0 {
                // Fuzzy match — Levenshtein against full word
                let len_diff = (word.len() as isize - tk.len() as isize).unsigned_abs();
                if len_diff <= max_dist {
                    let dist = levenshtein(tk, word);
                    if dist > 0 && dist <= max_dist {
                        let score = match dist {
                            1 => 0.5,
                            2 => 0.25,
                            _ => 0.1,
                        };
                        best = best.max(score);
                    }
                }
            }
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

    // Heading boost: score tokens against heading words using same position+coverage logic
    // Also split camelCase/hyphenated heading words (e.g. "CodeBlock" → ["code","block"])
    if !heading_words.is_empty() {
        let mut expanded_heading: Vec<String> = Vec::new();
        for hw in &heading_words {
            expanded_heading.push(hw.to_string());
            let parts = oxidoc_text::split_camel_case(hw);
            if parts.len() > 1 {
                for p in parts {
                    let lower = p.to_lowercase();
                    if lower.len() > 1 {
                        expanded_heading.push(lower);
                    }
                }
            }
        }

        let mut heading_score = 0.0_f32;
        let mut heading_hits = 0usize;
        for (i, tk) in tokens.iter().enumerate() {
            let raw = raw_tokens.get(i).map(|s| s.as_str()).unwrap_or(tk.as_str());
            let mut best = 0.0_f32;
            for hw in &expanded_heading {
                if oxidoc_text::stem(hw) == *tk {
                    best = 1.0;
                    break;
                }
                let score = word_match_score(hw, tk, raw);
                best = best.max(score);
            }
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

/// Compute phrase boost based on positional data.
///
/// For consecutive query token pairs, checks if any matched postings have
/// sequential positions (pos_b == pos_a + 1).
/// - All consecutive pairs sequential → 5x boost
/// - Partial → proportional boost
pub(super) fn compute_phrase_boost(
    query_tokens: &[String],
    token_postings: &[Vec<(&str, &[Posting])>],
    doc_id: u32,
) -> f32 {
    if query_tokens.len() < 2 {
        return 1.0;
    }

    let num_pairs = query_tokens.len() - 1;
    let mut sequential_pairs = 0;

    for pair_idx in 0..num_pairs {
        let postings_a = &token_postings[pair_idx];
        let postings_b = &token_postings[pair_idx + 1];

        let mut found_sequential = false;
        'outer: for (_key_a, posts_a) in postings_a {
            for post_a in posts_a.iter().filter(|p| p.doc_id == doc_id) {
                for (_key_b, posts_b) in postings_b {
                    for post_b in posts_b.iter().filter(|p| p.doc_id == doc_id) {
                        for &pos_a in &post_a.positions {
                            for &pos_b in &post_b.positions {
                                if pos_b == pos_a + 1 {
                                    found_sequential = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        if found_sequential {
            sequential_pairs += 1;
        }
    }

    if sequential_pairs == 0 {
        return 1.0;
    }

    let ratio = sequential_pairs as f32 / num_pairs as f32;
    // Full phrase match = 5x, partial = proportional
    1.0 + ratio * 4.0
}

/// Get the text for the section containing the given offset (from heading to next heading).
pub(super) fn get_section_text<'a>(
    text: &'a str,
    headings: &[crate::types::HeadingPos],
    offset: usize,
) -> &'a str {
    // If offset is before the first heading, return intro text only
    if headings.is_empty() || offset < headings[0].offset {
        let end = headings.first().map(|h| h.offset).unwrap_or(text.len());
        return &text[0..end];
    }
    let mut section_start = 0;
    let mut section_end = text.len();
    for (i, h) in headings.iter().enumerate() {
        if h.offset <= offset {
            section_start = h.offset;
            section_end = headings
                .get(i + 1)
                .map(|next| next.offset)
                .unwrap_or(text.len());
        } else {
            break;
        }
    }
    &text[section_start..section_end]
}

/// Given a match offset, walk backwards through heading positions to build breadcrumb.
/// Returns (breadcrumb, anchor) where breadcrumb is e.g. ["Page Title", "h2"]
/// and anchor is the closest heading's anchor.
pub(super) fn resolve_heading_breadcrumb(
    page_title: &str,
    headings: &[crate::types::HeadingPos],
    match_offset: Option<usize>,
) -> (Vec<String>, String) {
    let offset = match match_offset {
        Some(o) => o,
        None => return (vec![], String::new()),
    };

    // Find the last heading whose offset <= match offset
    let mut closest_idx: Option<usize> = None;
    for (i, h) in headings.iter().enumerate() {
        if h.offset <= offset {
            closest_idx = Some(i);
        } else {
            break;
        }
    }

    let closest_idx = match closest_idx {
        Some(i) => i,
        None => return (vec![], String::new()), // Match is before any heading
    };

    let closest = &headings[closest_idx];
    let anchor = closest.anchor.clone();

    // Build breadcrumb: page title, then ancestor headings (lower depth), then closest
    let mut crumbs = vec![page_title.to_string()];
    let closest_depth = closest.depth;

    // Walk backwards to find ancestor headings
    let mut need_depth = closest_depth - 1;
    let mut ancestors = Vec::new();
    for i in (0..closest_idx).rev() {
        if headings[i].depth <= need_depth {
            ancestors.push(headings[i].title.clone());
            if headings[i].depth <= 2 {
                break;
            }
            need_depth = headings[i].depth - 1;
        }
    }
    ancestors.reverse();
    crumbs.extend(ancestors);
    crumbs.push(closest.title.clone());

    (crumbs, anchor)
}
