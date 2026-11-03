//! First match of the matched terms in each section of a document.

use super::matching::find_bytes_from;
use crate::types::HeadingPos;

/// One term and how far its left-to-right, non-overlapping scan has gone.
struct TermCursor<'a> {
    /// Term bytes. Never empty.
    term: &'a [u8],
    /// True when a proper prefix of the term equals a suffix, so matches can overlap.
    bordered: bool,
    /// The next scan match is at or after this offset.
    pos: usize,
    /// The next scan match, once a search found it.
    found: Option<usize>,
    /// True when the scan has no match left.
    done: bool,
}

impl<'a> TermCursor<'a> {
    fn new(term: &'a [u8]) -> Self {
        let len = term.len();
        let bordered = (1..len).any(|k| term[..k] == term[len - k..]);
        Self {
            term,
            bordered,
            pos: 0,
            found: None,
            done: false,
        }
    }

    /// Returns the first scan match at or after `lo` when it starts before `limit`.
    ///
    /// The scan resumes after the end of each match. Without a border, matches never
    /// overlap, so every occurrence is a scan match and the search can skip to `lo`.
    /// A search stops at `limit`, so a later call with a higher limit resumes there.
    fn first_before(&mut self, hay: &[u8], lo: usize, limit: usize) -> Option<usize> {
        loop {
            if let Some(at) = self.found {
                if at >= lo {
                    return (at < limit).then_some(at);
                }
                self.found = None;
                self.pos = if self.bordered {
                    at + self.term.len()
                } else {
                    lo
                };
                continue;
            }
            if self.done {
                return None;
            }
            let start = if self.bordered {
                self.pos
            } else {
                self.pos.max(lo)
            };
            if start >= limit {
                return None;
            }
            // Only a match that starts before `limit` matters to this call.
            let end = limit.saturating_add(self.term.len() - 1).min(hay.len());
            match hay
                .get(..end)
                .and_then(|h| find_bytes_from(h, self.term, start))
            {
                Some(at) => self.found = Some(at),
                None => {
                    self.pos = limit;
                    self.done = end == hay.len();
                    return None;
                }
            }
        }
    }
}

/// Returns the first match offset in each section that holds a match, in text order.
///
/// A match is an occurrence of a term in `lower` found by scanning left to right and
/// resuming after each match. Each entry pairs the offset with the section heading:
/// the last heading at or before the offset, scanning headings in order and stopping
/// at the first heading past it, or `None` before every heading. Empty terms match
/// nothing.
pub(super) fn first_match_per_section(
    lower: &str,
    terms: &[&str],
    headings: &[HeadingPos],
) -> Vec<(usize, Option<usize>)> {
    let hay = lower.as_bytes();
    let mut cursors: Vec<TermCursor<'_>> = terms
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| TermCursor::new(t.as_bytes()))
        .collect();
    let mut firsts = Vec::new();
    if cursors.is_empty() {
        return firsts;
    }

    // Heading i owns offsets in [max(h_0..=h_i), max(h_0..=h_{i+1})).
    let mut starts = Vec::with_capacity(headings.len());
    let mut prefix_max = 0;
    for h in headings {
        prefix_max = prefix_max.max(h.offset);
        starts.push(prefix_max);
    }
    let regions =
        std::iter::once((None, 0)).chain(starts.iter().enumerate().map(|(i, &s)| (Some(i), s)));
    for (heading, lo) in regions {
        let next_index = heading.map_or(0, |i| i + 1);
        let hi = starts.get(next_index).copied().unwrap_or(usize::MAX);
        if lo >= hi {
            continue;
        }
        // Each match found lowers the limit, so later terms search a shorter span.
        let mut first: Option<usize> = None;
        let mut limit = hi;
        for cursor in &mut cursors {
            if let Some(at) = cursor.first_before(hay, lo, limit) {
                first = Some(at);
                limit = at;
            }
        }
        cursors.retain(|c| !c.done || c.found.is_some());
        if let Some(at) = first {
            firsts.push((at, heading));
        }
        if cursors.is_empty() {
            break;
        }
    }
    firsts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the index of the heading whose section holds `offset`.
    ///
    /// That is the last heading at or before `offset`, scanning in order and stopping at
    /// the first heading past it. Returns `None` when `offset` lies before every heading.
    fn closest_heading(headings: &[HeadingPos], offset: usize) -> Option<usize> {
        let mut closest = None;
        for (i, h) in headings.iter().enumerate() {
            if h.offset <= offset {
                closest = Some(i);
            } else {
                break;
            }
        }
        closest
    }

    /// Reference: every scan match of every term, sorted, then the first per section.
    fn reference(
        lower: &str,
        terms: &[&str],
        headings: &[HeadingPos],
    ) -> Vec<(usize, Option<usize>)> {
        let mut offsets = Vec::new();
        for term in terms.iter().filter(|t| !t.is_empty()) {
            let mut start = 0;
            while let Some(idx) = lower[start..].find(term) {
                offsets.push(start + idx);
                start += idx + term.len();
            }
        }
        offsets.sort();
        offsets.dedup();
        let mut firsts: Vec<(usize, Option<usize>)> = Vec::new();
        for offset in offsets {
            let heading = closest_heading(headings, offset);
            if firsts.last().is_none_or(|&(_, h)| h != heading) {
                firsts.push((offset, heading));
            }
        }
        firsts
    }

    fn heading(offset: usize) -> HeadingPos {
        HeadingPos {
            title: format!("h{offset}"),
            anchor: format!("a{offset}"),
            depth: 2,
            offset,
        }
    }

    #[test]
    fn matches_reference() {
        let text = "aaaa tensor abab ababab tensors kernel aaa über tensor ab";
        let term_sets: [&[&str]; 6] = [
            &["tensor"],
            &["aa"],
            &["abab", "tensor"],
            &["aaa", "kernel", ""],
            &["zzz"],
            &["ab", "b", "über", "r"],
        ];
        let heading_sets: [Vec<usize>; 7] = [
            vec![],
            vec![0],
            vec![5, 17, 40],
            vec![12, 12, 30],
            vec![30, 10, 45],
            vec![3, 4, 6, 8, 11, 13, 15, 52],
            vec![100],
        ];
        for terms in term_sets {
            for offsets in &heading_sets {
                let headings: Vec<HeadingPos> = offsets.iter().map(|&o| heading(o)).collect();
                assert_eq!(
                    first_match_per_section(text, terms, &headings),
                    reference(text, terms, &headings),
                    "{terms:?} / {offsets:?}"
                );
            }
        }
    }

    #[test]
    fn matches_reference_on_generated_text() {
        // Linear congruential generator: a fixed, dependency-free input stream.
        let mut state: u32 = 12345;
        let mut next = move |n: u32| -> u32 {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (state >> 16) % n
        };
        let alphabet = ['a', 'b', ' ', 'é'];
        let term_sets: [&[&str]; 5] = [
            &["a"],
            &["ab", "aba"],
            &["aa", "b a"],
            &["abab", "é", "bé"],
            &["aaa", "bb", "ab a"],
        ];
        for _ in 0..300 {
            let len = next(60) as usize;
            let text: String = (0..len).map(|_| alphabet[next(4) as usize]).collect();
            let mut offsets: Vec<usize> = (0..next(5))
                .map(|_| next(len as u32 + 5) as usize)
                .collect();
            if next(2) == 0 {
                offsets.sort_unstable();
            }
            let headings: Vec<HeadingPos> = offsets.iter().map(|&o| heading(o)).collect();
            for terms in term_sets {
                assert_eq!(
                    first_match_per_section(&text, terms, &headings),
                    reference(&text, terms, &headings),
                    "{text:?} / {terms:?} / {offsets:?}"
                );
            }
        }
    }
}
