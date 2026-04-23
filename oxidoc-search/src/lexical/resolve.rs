use crate::types::Posting;
use std::collections::{HashMap, HashSet};

use super::matching::find_matching_postings;

/// Result of resolving query tokens against the posting index.
pub(super) struct ResolvedQuery<'a> {
    pub candidate_docs: HashSet<u32>,
    pub use_and: bool,
    pub per_token_doc_ids: Vec<HashSet<u32>>,
    pub doc_fuzzy_keys: HashMap<u32, Vec<String>>,
    pub doc_matched_keys: HashMap<u32, Vec<String>>,
    pub token_postings_for_phrase: Vec<Vec<(&'a str, &'a [Posting])>>,
}

/// Resolve query tokens against a postings index: find candidates, collect
/// matched/fuzzy keys per document, and prepare phrase-boost data.
pub(super) fn resolve_tokens<'a>(
    tokens: &[String],
    raw_tokens: &[String],
    postings: &'a HashMap<String, Vec<Posting>>,
) -> ResolvedQuery<'a> {
    let posting_keys: Vec<&String> = postings.keys().collect();
    let mut token_matches: Vec<Vec<(&str, f32, bool)>> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let mut matches = find_matching_postings(token, &posting_keys);
        // Also try the raw (unstemmed) form — partial queries like "analy"
        // stem to "anali" but still prefix-match stemmed postings like "analyt".
        if let Some(raw) = raw_tokens.get(i)
            && raw != token
        {
            for m in find_matching_postings(raw, &posting_keys) {
                if !matches.iter().any(|existing| existing.0 == m.0) {
                    matches.push(m);
                }
            }
        }
        token_matches.push(matches);
    }

    // Collect doc IDs per token for AND logic
    let mut per_token_doc_ids: Vec<HashSet<u32>> = Vec::new();
    for matches in &token_matches {
        let mut doc_ids: HashSet<u32> = HashSet::new();
        for (key, _, _) in matches {
            if let Some(posts) = postings.get(*key) {
                for posting in posts {
                    doc_ids.insert(posting.doc_id);
                }
            }
        }
        per_token_doc_ids.push(doc_ids);
    }

    // AND logic: intersect all token doc sets
    let mut candidate_docs: HashSet<u32> = if per_token_doc_ids.is_empty() {
        HashSet::new()
    } else {
        per_token_doc_ids[0].clone()
    };
    for doc_ids in &per_token_doc_ids[1..] {
        candidate_docs = candidate_docs.intersection(doc_ids).copied().collect();
    }

    // If AND yields 0 results, fall back to OR with penalty
    let use_and = !candidate_docs.is_empty();
    if !use_and {
        for doc_ids in &per_token_doc_ids {
            candidate_docs.extend(doc_ids);
        }
    }

    // Collect fuzzy keys per doc (for highlight_terms)
    let mut doc_fuzzy_keys: HashMap<u32, Vec<String>> = HashMap::new();
    for matches in &token_matches {
        for (key, _, is_fuzzy) in matches {
            if *is_fuzzy && let Some(posts) = postings.get(*key) {
                for posting in posts {
                    doc_fuzzy_keys
                        .entry(posting.doc_id)
                        .or_default()
                        .push(key.to_string());
                }
            }
        }
    }

    // Collect all matched posting keys per doc (for text offset finding)
    let mut doc_matched_keys: HashMap<u32, Vec<String>> = HashMap::new();
    for matches in &token_matches {
        for (key, _, _) in matches {
            if let Some(posts) = postings.get(*key) {
                for posting in posts {
                    doc_matched_keys
                        .entry(posting.doc_id)
                        .or_default()
                        .push(key.to_string());
                }
            }
        }
    }

    // Build postings data for phrase boost
    let token_postings_for_phrase: Vec<Vec<(&str, &[Posting])>> = token_matches
        .iter()
        .map(|matches| {
            matches
                .iter()
                .filter_map(|(key, _, _)| postings.get(*key).map(|posts| (*key, posts.as_slice())))
                .collect()
        })
        .collect();

    ResolvedQuery {
        candidate_docs,
        use_and,
        per_token_doc_ids,
        doc_fuzzy_keys,
        doc_matched_keys,
        token_postings_for_phrase,
    }
}
