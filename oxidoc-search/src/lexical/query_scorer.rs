//! Per-query word scores, computed once per vocabulary word and token.

use super::levenshtein::Levenshtein;
use super::matching::{find_bytes, max_edit_distance};
use super::vocab::Vocab;

/// Scores of vocabulary words against the tokens of one query.
///
/// A word score depends only on the word, its stem and the token, so each score is
/// computed once per query and reused across sections and documents.
pub(super) struct QueryScorer<'a> {
    vocab: &'a Vocab,
    tokens: Vec<TokenMemo<'a>>,
    lev: Levenshtein,
}

/// Forms of one query token and its memoised word scores.
struct TokenMemo<'a> {
    /// Stemmed token.
    tk: &'a str,
    /// Lowercased token before stemming.
    raw: &'a str,
    /// Body score per word id. NaN marks a score not computed yet.
    body: Vec<f32>,
    /// Heading score per word id. NaN marks a score not computed yet.
    heading: Vec<f32>,
}

impl<'a> QueryScorer<'a> {
    /// Creates a scorer for `tokens`. A token without a raw form uses its stem.
    pub(super) fn new(vocab: &'a Vocab, tokens: &'a [String], raw_tokens: &'a [String]) -> Self {
        let tokens = tokens
            .iter()
            .enumerate()
            .map(|(i, tk)| TokenMemo {
                tk: tk.as_str(),
                raw: raw_tokens.get(i).map_or(tk.as_str(), String::as_str),
                body: vec![f32::NAN; vocab.len()],
                heading: vec![f32::NAN; vocab.len()],
            })
            .collect();
        Self {
            vocab,
            tokens,
            lev: Levenshtein::default(),
        }
    }

    /// Returns the number of query tokens.
    pub(super) fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Returns the best body score of token `t` over the words `ids`.
    ///
    /// A stem match scores 1.0. Other matches score below 1.0.
    pub(super) fn best_body(&mut self, t: usize, ids: &[usize]) -> f32 {
        let mut best = 0.0_f32;
        for &id in ids {
            let score = self.body_score(t, id);
            if score >= 1.0 {
                return 1.0;
            }
            best = best.max(score);
        }
        best
    }

    /// Returns the best heading score of token `t` over the words `ids`.
    pub(super) fn best_heading(&mut self, t: usize, ids: &[usize]) -> f32 {
        let mut best = 0.0_f32;
        for &id in ids {
            let score = self.heading_score(t, id);
            if score >= 1.0 {
                return 1.0;
            }
            best = best.max(score);
        }
        best
    }

    /// Returns true when some word of `ids` covers token `t` as a title word.
    ///
    /// A word covers the token when its stem equals the token or it starts with
    /// either token form.
    pub(super) fn covers(&self, t: usize, ids: &[usize]) -> bool {
        let Some(memo) = self.tokens.get(t) else {
            return false;
        };
        ids.iter().any(|&id| {
            let (lower, stem) = self.vocab.get(id);
            stem == memo.tk || lower.starts_with(memo.tk) || lower.starts_with(memo.raw)
        })
    }

    /// Body score of word `id` for token `t`: stem, substring, then fuzzy match.
    fn body_score(&mut self, t: usize, id: usize) -> f32 {
        let Some(memo) = self.tokens.get_mut(t) else {
            return 0.0;
        };
        let Some(slot) = memo.body.get_mut(id) else {
            return 0.0;
        };
        if !slot.is_nan() {
            return *slot;
        }
        let (word, stem) = self.vocab.get(id);
        let (tk, raw) = (memo.tk, memo.raw);
        let score = if stem == tk {
            1.0
        } else {
            let score = word_match_score(word, tk, raw);
            let max_dist = max_edit_distance(tk.len());
            if score > 0.0 {
                score
            } else if max_dist > 0 && word.len().abs_diff(tk.len()) <= max_dist {
                // Fuzzy match — Levenshtein against full word
                match self.lev.within(tk, word, max_dist) {
                    Some(1) => 0.5,
                    Some(2) => 0.25,
                    Some(dist) if dist > 0 => 0.1,
                    _ => 0.0,
                }
            } else {
                0.0
            }
        };
        *slot = score;
        score
    }

    /// Heading score of word `id` for token `t`: stem, then substring match.
    fn heading_score(&mut self, t: usize, id: usize) -> f32 {
        let Some(memo) = self.tokens.get_mut(t) else {
            return 0.0;
        };
        let Some(slot) = memo.heading.get_mut(id) else {
            return 0.0;
        };
        if !slot.is_nan() {
            return *slot;
        }
        let (word, stem) = self.vocab.get(id);
        let score = if stem == memo.tk {
            1.0
        } else {
            word_match_score(word, memo.tk, memo.raw)
        };
        *slot = score;
        score
    }
}

/// Score a word against the (stem, raw) forms of a token.
///
/// A token that's a prefix of the word (pos=0) scores 0.9 regardless of
/// word length — a typed prefix like "ve" should match "versioning" as
/// strongly as it matches "vercel", so the ranking is decided by other
/// signals (title match, heading boost) rather than whichever word happens
/// to be shorter. Non-prefix substring matches fall back to the original
/// position×coverage formula to keep mid-word matches weaker.
fn word_match_score(word: &str, tk: &str, raw: &str) -> f32 {
    let mut best = 0.0_f32;
    for form in [tk, raw] {
        if let Some(pos) = find_bytes(word, form) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexical::vocab::VocabBuilder;

    #[test]
    fn word_scores_table() {
        let mut builder = VocabBuilder::default();
        let ids = builder.distinct_ids(["blocks", "codeblock", "glock", "tensor"]);
        let vocab = builder.finish();
        let (tokens, raws): (Vec<String>, Vec<String>) =
            oxidoc_text::tokenize_query("block").into_iter().unzip();
        let mut scorer = QueryScorer::new(&vocab, &tokens, &raws);
        let body: Vec<f32> = ids.iter().map(|&id| scorer.best_body(0, &[id])).collect();
        let heading: Vec<f32> = ids
            .iter()
            .map(|&id| scorer.best_heading(0, &[id]))
            .collect();
        let mid = (1.0 - 4.0 / 9.0) * (5.0 / 9.0);
        assert_eq!(body, [1.0, mid, 0.5, 0.0]);
        assert_eq!(heading, [1.0, mid, 0.0, 0.0]);
        assert_eq!(scorer.best_body(0, &ids), 1.0);
        assert!(scorer.covers(0, &ids));
        assert!(!scorer.covers(0, &ids[1..]));
    }
}
