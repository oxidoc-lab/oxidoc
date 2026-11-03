# oxidoc-search

Lexical and semantic search for static sites. A native builder writes index files. A wasm runtime reads them in the browser.

- Lexical search uses BM25 over a chunked index. The browser fetches only the chunks a query needs.
- Semantic search embeds text with a GGUF model and ranks int8 vectors. Hybrid queries fuse both rankings with reciprocal rank fusion.
- Records can carry field weights, a boost, and tags for filters and facets.

## Features

| Feature | Default | Adds |
| --- | --- | --- |
| `wasm` | yes | `wasm-bindgen` exports and the `SemanticSearch` island |
| `semantic` | yes | Embedding search, vector decoding, hybrid ranking |
| `builder` | no | Native lexical index builder: `oxidoc_search::builder` |
| `semantic-build` | no | Vector builder that embeds records on the CPU. Implies `builder` and `semantic` |

## Build an index

Enable `builder`. Pass `Record` values to `build_lexical` and `build_records`. Write the result with `write_lexical`.

`encode_lexical` returns the same files in memory as `IndexFile` values, each with a `name` and `bytes`. It touches no disk.

```rust
use oxidoc_search::builder::{build_lexical, build_records, write_lexical};
use oxidoc_search::{Field, Record};

let mut intro = Record::new("intro", "/docs/intro", "Introduction");
intro.fields.push(Field::new("body", "Oxidoc builds static documentation sites."));
intro.tags.push(("kind".to_string(), "guide".to_string()));

let mut search = Record::new("search", "/docs/search", "Search");
search.fields.push(Field::new("body", "Search runs in the browser with wasm."));

let records = vec![intro, search];
let index = build_lexical(&records);
let extras = build_records(&records);

let dir = std::env::temp_dir().join("oxidoc-search-readme-example");
std::fs::create_dir_all(&dir).unwrap();
write_lexical(&index, Some(&extras), &dir).unwrap();

assert!(dir.join("search-meta.bin").exists());
assert!(dir.join("search-records.bin").exists());
std::fs::remove_dir_all(&dir).unwrap();
```

Doc ids follow input order. The same input produces the same bytes.

## Output files

| File | Content |
| --- | --- |
| `search-meta.bin` | Documents and the chunk manifest |
| `search-chunk-N.bin` | Postings of the terms that share one 2-character prefix |
| `search-records.bin` | Field weights, boosts, and tags. Written only when a record uses them |
| `search-vectors.bin` | int8 embeddings. Written by `semantic-build` |
| `search-model.gguf` | Embedding model for the browser. Copied by `semantic-build` |

## Browser exports

With `wasm`, the module exports these functions:

- `oxidoc_search_init(data)`
- `oxidoc_search_needed_chunks(query)`
- `oxidoc_search_load_chunk(data)`
- `oxidoc_search_load_records(data)`
- `oxidoc_search_load_semantic(model_data, vectors_bin) (`semantic` only)
- `oxidoc_search_query(query_text, max_results)`
- `oxidoc_search_query_filtered(query_text, max_results, filters_json)`
- `oxidoc_search_facets(query_text, filters_json)`
- `oxidoc_search_query_ai(query_text, max_results) (`semantic` only)
- `oxidoc_search_query_ai_filtered(query_text, max_results, filters_json) (`semantic` only)
- `oxidoc_search_ready()`
- `oxidoc_search_has_semantic()`

Fetch `search-meta.bin` and call `oxidoc_search_init`. For each query, call `oxidoc_search_needed_chunks`, fetch the listed chunks, and pass them to `oxidoc_search_load_chunk`.

## License

Apache-2.0
