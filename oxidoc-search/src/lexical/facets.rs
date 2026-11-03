//! Facet counts for a query.

use std::collections::{BTreeMap, BTreeSet};

use super::search::LexicalSearcher;
use crate::records::Facets;
use crate::records::filter::{Filter, matches_filters_except};

impl LexicalSearcher {
    /// Counts tag values over the documents that match `query`.
    ///
    /// A document counts once per distinct tag value. Counts for one key apply
    /// every filter except the filter on that key, so the other values of a
    /// selected key keep their counts. An empty query counts every document.
    /// Keys come in sorted order. Values come by count, highest first, then by
    /// value. Without records the result is empty.
    pub fn facets(&self, query: &str, filters: &[Filter]) -> Facets {
        let Some(records) = &self.records else {
            return Vec::new();
        };
        let matched: BTreeSet<u32> = if query.trim().is_empty() {
            self.documents.iter().map(|d| d.id).collect()
        } else {
            self.search_hits(query, &[])
                .iter()
                .map(|hit| hit.doc_id)
                .collect()
        };

        let mut counts: BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();
        for doc_id in matched {
            let Some(record) = records.get(doc_id as usize) else {
                continue;
            };
            let pairs: BTreeSet<(&str, &str)> = record
                .tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            for (key, value) in pairs {
                if matches_filters_except(&record.tags, filters, Some(key)) {
                    *counts.entry(key).or_default().entry(value).or_insert(0) += 1;
                }
            }
        }

        counts
            .into_iter()
            .map(|(key, values)| {
                let mut values: Vec<(String, usize)> = values
                    .into_iter()
                    .map(|(value, count)| (value.to_string(), count))
                    .collect();
                values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                (key.to_string(), values)
            })
            .collect()
    }
}
