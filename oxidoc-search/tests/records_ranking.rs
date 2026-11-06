//! Ranking, filters and facets over built index files with a records side file.
#![cfg(feature = "builder")]

use std::path::Path;

use oxidoc_search::builder::write::META_FILE;
use oxidoc_search::builder::{build_lexical, build_records, write_lexical};
use oxidoc_search::records::RECORDS_FILE;
use oxidoc_search::types::HeadingPos;
use oxidoc_search::{Facets, Field, Filter, LexicalSearcher, Record, SearchQuery};

/// Catalogue field weights: name > keywords > description > readme.
const WEIGHTS: [(&str, f32); 4] = [
    ("name", 4.0),
    ("keywords", 3.0),
    ("description", 2.0),
    ("readme", 1.0),
];

/// Builds one catalogue record. The title and the `name` field are the project name.
/// `texts` holds the keywords, description and readme.
fn project(name: &str, texts: [&str; 3], tags: &[(&str, &str)], boost: f32) -> Record {
    let mut record = Record::new(name, format!("/p/{name}"), name);
    let texts = [name, texts[0], texts[1], texts[2]];
    for ((field, weight), text) in WEIGHTS.iter().zip(texts) {
        record.fields.push(Field {
            weight: *weight,
            ..Field::new(*field, text)
        });
    }
    record.tags = tags
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    record.boost = boost;
    record
}

/// Builds the page-path record: one `body` field, weight 1.0, no tags.
fn page(title: &str, slug: &str, text: &str, headings: Vec<HeadingPos>) -> Record {
    let mut record = Record::new(slug, format!("/{slug}"), title);
    record.fields.push(Field::new("body", text));
    record.headings = headings;
    record
}

/// Loads the metadata, every chunk, and the records file when present.
fn load(dir: &Path) -> LexicalSearcher {
    let meta = std::fs::read(dir.join(META_FILE)).unwrap();
    let mut searcher = LexicalSearcher::from_metadata(&meta).unwrap();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with("search-chunk-") {
            searcher.load_chunk(&std::fs::read(&path).unwrap()).unwrap();
        }
    }
    let records = dir.join(RECORDS_FILE);
    if records.exists() {
        searcher
            .load_records(&std::fs::read(&records).unwrap())
            .unwrap();
    }
    searcher
}

/// Builds and writes the index files for `records`, then loads them.
fn site(records: &[Record]) -> (tempfile::TempDir, LexicalSearcher) {
    let dir = tempfile::tempdir().unwrap();
    let extras = build_records(records);
    write_lexical(&build_lexical(records), Some(&extras), dir.path()).unwrap();
    let searcher = load(dir.path());
    (dir, searcher)
}

fn filter(key: &str, values: &[&str]) -> Filter {
    (
        key.to_string(),
        values.iter().map(|v| v.to_string()).collect(),
    )
}

fn query(text: &str, filters: Vec<Filter>) -> SearchQuery {
    SearchQuery {
        text: text.to_string(),
        max_results: 50,
        filters,
    }
}

fn paths(searcher: &LexicalSearcher, text: &str, filters: Vec<Filter>) -> Vec<String> {
    searcher
        .search(&query(text, filters))
        .into_iter()
        .map(|r| r.path)
        .collect()
}

#[test]
fn field_weight_and_boost_orderings() {
    // Each case lists records in doc id order and the expected result order.
    // Earlier doc ids win ties, so every expected order needs the new signal.
    struct Case {
        name: &'static str,
        records: Vec<Record>,
        query: &'static str,
        want: Vec<&'static str>,
    }
    let neutral = |mut r: Record, title: &str| {
        r.title = title.to_string();
        r
    };
    let cases = vec![
        Case {
            name: "name match beats readme match",
            records: vec![
                neutral(
                    project("splitter", ["", "", "a tokenizer for text"], &[], 1.0),
                    "Project A",
                ),
                neutral(
                    project("tokenizer", ["", "", "fast text splitting"], &[], 1.0),
                    "Project B",
                ),
            ],
            query: "tokenizer",
            want: vec!["/p/tokenizer", "/p/splitter"],
        },
        Case {
            name: "keywords match beats description match",
            records: vec![
                project("alpha", ["", "parser crate", ""], &[], 1.0),
                project("beta", ["parser", "", ""], &[], 1.0),
            ],
            query: "parser",
            want: vec!["/p/beta", "/p/alpha"],
        },
        Case {
            name: "equal text, higher boost first",
            records: vec![
                project("alpha", ["", "vector search", ""], &[], 1.0),
                project("beta", ["", "vector search", ""], &[], 1.3),
            ],
            query: "vector",
            want: vec!["/p/beta", "/p/alpha"],
        },
        Case {
            name: "boost never beats an exact name match",
            records: vec![
                project("fastsplit", ["tokenizer", "", "tokenizer"], &[], 100.0),
                project("tokenizer", ["", "", "splits text"], &[], 0.01),
            ],
            query: "tokenizer",
            want: vec!["/p/tokenizer", "/p/fastsplit"],
        },
    ];
    for case in cases {
        let (_dir, searcher) = site(&case.records);
        assert_eq!(
            paths(&searcher, case.query, vec![]),
            case.want,
            "{}",
            case.name
        );
    }
}

#[test]
fn boost_never_beats_name_match_with_unit_weights() {
    let mut records = vec![
        project("fastsplit", ["", "", "tokenizer"], &[], 100.0),
        project("tokenizer", ["", "", "splits text"], &[], 0.01),
    ];
    for record in &mut records {
        for field in &mut record.fields {
            field.weight = 1.0;
        }
    }
    let (_dir, searcher) = site(&records);
    assert_eq!(
        paths(&searcher, "tokenizer", vec![]),
        vec!["/p/tokenizer", "/p/fastsplit"]
    );
}

/// Four libraries with the same text, so results follow doc id order.
fn tagged_catalogue() -> Vec<Record> {
    let texts = ["", "rust crate", ""];
    vec![
        project("a", texts, &[("kind", "library"), ("topic", "nlp")], 1.0),
        project("b", texts, &[("kind", "library"), ("topic", "vision")], 1.0),
        project("c", texts, &[("kind", "app"), ("topic", "nlp")], 1.0),
        project(
            "d",
            texts,
            &[("kind", "app"), ("topic", "audio"), ("topic", "nlp")],
            1.0,
        ),
    ]
}

#[test]
fn filters_or_within_key_and_across_keys() {
    let (_dir, searcher) = site(&tagged_catalogue());
    let cases = [
        (vec![], vec!["/p/a", "/p/b", "/p/c", "/p/d"]),
        (vec![filter("kind", &["library"])], vec!["/p/a", "/p/b"]),
        (
            vec![filter("topic", &["vision", "audio"])],
            vec!["/p/b", "/p/d"],
        ),
        (
            vec![filter("kind", &["app"]), filter("topic", &["nlp"])],
            vec!["/p/c", "/p/d"],
        ),
        (
            vec![filter("kind", &["library"]), filter("topic", &["audio"])],
            vec![],
        ),
        (vec![filter("license", &["mit"])], vec![]),
        (
            vec![filter("kind", &[])],
            vec!["/p/a", "/p/b", "/p/c", "/p/d"],
        ),
    ];
    for (filters, want) in cases {
        assert_eq!(
            paths(&searcher, "rust", filters.clone()),
            want,
            "{filters:?}"
        );
    }
}

#[test]
fn facet_counts_follow_query_and_other_keys() {
    let (_dir, searcher) = site(&tagged_catalogue());
    type Want = Vec<(&'static str, Vec<(&'static str, usize)>)>;
    let cases: [(&str, Vec<Filter>, Want); 4] = [
        (
            "rust",
            vec![],
            vec![
                ("kind", vec![("app", 2), ("library", 2)]),
                ("topic", vec![("nlp", 3), ("audio", 1), ("vision", 1)]),
            ],
        ),
        (
            "rust",
            vec![filter("kind", &["library"])],
            vec![
                ("kind", vec![("app", 2), ("library", 2)]),
                ("topic", vec![("nlp", 1), ("vision", 1)]),
            ],
        ),
        (
            "",
            vec![filter("topic", &["audio"])],
            vec![
                ("kind", vec![("app", 1)]),
                ("topic", vec![("nlp", 3), ("audio", 1), ("vision", 1)]),
            ],
        ),
        ("zzzzzz", vec![], vec![]),
    ];
    for (text, filters, want) in cases {
        let want: Facets = want
            .into_iter()
            .map(|(key, values)| {
                let values = values
                    .into_iter()
                    .map(|(value, count)| (value.to_string(), count))
                    .collect();
                (key.to_string(), values)
            })
            .collect();
        assert_eq!(
            searcher.facets(text, &filters),
            want,
            "{text:?} {filters:?}"
        );
    }
}

#[test]
fn page_only_index_is_unchanged() {
    let text = "Install the tool first. Configure the search index. Search runs in the browser.";
    let records = vec![
        page(
            "Search",
            "docs/search",
            text,
            vec![HeadingPos {
                title: "Configure".to_string(),
                anchor: "configure".to_string(),
                depth: 2,
                offset: 24,
            }],
        ),
        page(
            "Install",
            "docs/install",
            "Install with cargo. Search later.",
            vec![],
        ),
        page(
            "Index",
            "docs/index",
            "The index holds search terms.",
            vec![],
        ),
    ];
    let (dir, plain) = site(&records);
    assert!(
        !dir.path().join(RECORDS_FILE).exists(),
        "a page-only build must not write {RECORDS_FILE}"
    );
    assert!(!plain.has_records());

    // Force the records code path with the page-only records.
    let mut with_records = load(dir.path());
    with_records.set_records(build_records(&records)).unwrap();
    assert!(with_records.has_records());

    for text in [
        "search",
        "install",
        "configure index",
        "serch",
        "brow",
        "zzz",
    ] {
        let a = plain.search(&query(text, vec![]));
        let b = with_records.search(&query(text, vec![]));
        let render = |results: &[oxidoc_search::SearchDoc]| -> Vec<String> {
            results
                .iter()
                .map(|r| {
                    format!(
                        "{}|{}|{}|{:?}|{}|{:?}",
                        r.path,
                        r.title,
                        r.snippet,
                        r.breadcrumb,
                        r.score.to_bits(),
                        r.highlight_terms
                    )
                })
                .collect()
        };
        assert_eq!(render(&a), render(&b), "query {text:?}");
    }
    assert!(plain.facets("search", &[]).is_empty());
}

#[test]
fn filters_without_records_match_nothing() {
    let records = vec![page("Search", "search", "search text", vec![])];
    let (_dir, searcher) = site(&records);
    assert_eq!(paths(&searcher, "search", vec![]), vec!["/search"]);
    assert!(paths(&searcher, "search", vec![filter("kind", &["library"])]).is_empty());
}

#[test]
fn records_from_another_index_are_rejected() {
    let (dir, _) = site(&tagged_catalogue());
    let bytes = std::fs::read(dir.path().join(RECORDS_FILE)).unwrap();
    let (_other_dir, mut other) = site(&[page("One", "one", "text", vec![])]);
    assert!(other.load_records(&bytes).is_err());
    assert!(!other.has_records());
}

#[test]
fn results_carry_the_tags_of_their_record() {
    let records = tagged_catalogue();
    let (_dir, searcher) = site(&records);
    let results = searcher.search(&query("rust", vec![]));
    assert!(!results.is_empty());
    for result in &results {
        let record = records
            .iter()
            .find(|r| result.path.starts_with(&r.url))
            .unwrap();
        assert_eq!(result.tags, record.tags, "tags of {}", result.path);
    }
    assert!(results.iter().any(|r| !r.tags.is_empty()));
}

#[test]
fn page_only_results_serialize_without_a_tags_key() {
    let records = vec![page(
        "Search",
        "docs/search",
        "Search runs in the browser.",
        vec![],
    )];
    let (_dir, searcher) = site(&records);
    let results = searcher.search(&query("search", vec![]));
    assert!(!results.is_empty());
    for result in &results {
        assert!(result.tags.is_empty());
        let json = serde_json::to_string(result).unwrap();
        assert!(!json.contains("tags"), "unexpected tags key: {json}");
    }
    let json = serde_json::to_string(&results[0]).unwrap();
    assert_eq!(
        json,
        r#"{"title":"Search","path":"/docs/search","snippet":"Search runs in the browser.","score":6.0,"source":"Lexical","breadcrumb":[],"anchor":"","highlight_terms":[]}"#
    );
}

#[test]
fn a_camel_case_word_in_a_description_is_found_by_its_lowercase_form() {
    let records = [
        project(
            "numr",
            ["", "in the role NumPy has in Python", ""],
            &[],
            1.0,
        ),
        project("other", ["", "a plain library", ""], &[], 1.0),
    ];
    let (_dir, searcher) = site(&records);
    assert_eq!(paths(&searcher, "numpy", vec![]), ["/p/numr"]);
    assert_eq!(paths(&searcher, "NumPy", vec![]), ["/p/numr"]);
    assert_eq!(paths(&searcher, "num", vec![]), ["/p/numr"]);
}

#[test]
fn a_word_whose_stem_is_not_in_the_text_is_still_found() {
    let records = [
        project("one", ["", "a sparse matrix library", ""], &[], 1.0),
        project("two", ["", "many sparse libraries", ""], &[], 1.0),
        project("three", ["", "a plain crate", ""], &[], 1.0),
    ];
    let (_dir, searcher) = site(&records);
    for query in ["library", "libraries"] {
        let mut found = paths(&searcher, query, vec![]);
        found.sort();
        assert_eq!(found, ["/p/one", "/p/two"], "{query}");
    }
}

#[test]
fn a_stem_missing_from_the_text_finds_the_section_that_holds_the_word() {
    let text = "Intro about nothing.\nSection about entropy here.";
    let offset = text.find("Section").unwrap();
    let headings = vec![HeadingPos {
        offset,
        title: "Section".to_string(),
        anchor: "section".to_string(),
        depth: 2,
    }];
    let (_dir, searcher) = site(&[page("Doc", "doc", text, headings)]);
    assert_eq!(paths(&searcher, "entropy", vec![]), ["/doc#section"]);
    assert_eq!(paths(&searcher, "entropies", vec![]), ["/doc#section"]);
}
