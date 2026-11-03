//! Tag filters: OR within a key, AND across keys.

use std::collections::BTreeMap;

use crate::error::{SearchError, SearchResult};

/// One filter: a tag key and the values it accepts.
///
/// A document passes when it has the key with any of the values.
/// An empty value list accepts every document.
pub type Filter = (String, Vec<String>);

/// Facet counts: per tag key, the values and their document counts.
pub type Facets = Vec<(String, Vec<(String, usize)>)>;

/// Returns true when `tags` passes every filter in `filters`.
pub fn matches_filters(tags: &[(String, String)], filters: &[Filter]) -> bool {
    matches_filters_except(tags, filters, None)
}

/// Returns true when `tags` passes every filter whose key is not `skip`.
pub(crate) fn matches_filters_except(
    tags: &[(String, String)],
    filters: &[Filter],
    skip: Option<&str>,
) -> bool {
    filters.iter().all(|(key, values)| {
        skip == Some(key.as_str())
            || values.is_empty()
            || tags
                .iter()
                .any(|(k, v)| k == key && values.iter().any(|want| want == v))
    })
}

/// Returns true when no filter constrains anything.
pub(crate) fn filters_are_empty(filters: &[Filter]) -> bool {
    filters.iter().all(|(_, values)| values.is_empty())
}

/// Parses filters from JSON such as `{"kind":["library"],"topic":["nlp","vision"]}`.
///
/// Keys come back in sorted order. Empty or whitespace-only input means no filters.
/// Any other shape returns [`SearchError::InvalidQuery`].
pub fn parse_filters_json(json: &str) -> SearchResult<Vec<Filter>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let map: BTreeMap<String, Vec<String>> = serde_json::from_str(json).map_err(|e| {
        SearchError::InvalidQuery(format!(
            "filters must be a JSON object of string arrays, like {{\"kind\":[\"library\"]}}: {e}"
        ))
    })?;
    Ok(map.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn filter(key: &str, values: &[&str]) -> Filter {
        (
            key.to_string(),
            values.iter().map(|v| v.to_string()).collect(),
        )
    }

    #[test]
    fn matches_filters_table() {
        let doc = tags(&[("kind", "library"), ("topic", "nlp"), ("topic", "audio")]);
        let cases = [
            (vec![], true),
            (vec![filter("kind", &["library"])], true),
            (vec![filter("kind", &["app"])], false),
            (vec![filter("kind", &["app", "library"])], true),
            (vec![filter("topic", &["vision", "audio"])], true),
            (
                vec![filter("kind", &["library"]), filter("topic", &["nlp"])],
                true,
            ),
            (
                vec![filter("kind", &["library"]), filter("topic", &["vision"])],
                false,
            ),
            (vec![filter("license", &["mit"])], false),
            (vec![filter("license", &[])], true),
        ];
        for (filters, want) in cases {
            assert_eq!(matches_filters(&doc, &filters), want, "{filters:?}");
        }
    }

    #[test]
    fn skip_ignores_one_key() {
        let doc = tags(&[("kind", "library")]);
        let filters = vec![filter("kind", &["app"])];
        assert!(!matches_filters(&doc, &filters));
        assert!(matches_filters_except(&doc, &filters, Some("kind")));
    }

    #[test]
    fn parse_filters_json_table() {
        let ok = [
            ("", vec![]),
            ("  ", vec![]),
            ("{}", vec![]),
            (
                r#"{"topic":["nlp","vision"],"kind":["library"]}"#,
                vec![
                    filter("kind", &["library"]),
                    filter("topic", &["nlp", "vision"]),
                ],
            ),
        ];
        for (json, want) in ok {
            assert_eq!(parse_filters_json(json).unwrap(), want, "{json}");
        }
        for json in [
            "{",
            "[]",
            r#"{"kind":"library"}"#,
            r#"{"kind":[1]}"#,
            "null",
        ] {
            assert!(
                matches!(parse_filters_json(json), Err(SearchError::InvalidQuery(_))),
                "{json} must be rejected"
            );
        }
    }
}
