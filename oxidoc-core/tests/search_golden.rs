//! Golden test for the lexical search index.
//!
//! Builds the index from `tests/fixtures/search-golden` and byte-compares every
//! output file with `tests/golden/search`. Set `OXIDOC_UPDATE_GOLDEN=1` to rewrite
//! the golden files from the current build.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use oxidoc_core::config::load_config;
use oxidoc_core::crawler::discover_pages;
use oxidoc_core::search_index::extract::extract_page_text;
use oxidoc_core::search_index::lexical::{build_lexical_index, write_lexical_index};
use oxidoc_core::search_index::{ChunkPostings, SearchMetadata};

const UPDATE_ENV: &str = "OXIDOC_UPDATE_GOLDEN";
const RERUN: &str = "rerun with `OXIDOC_UPDATE_GOLDEN=1 cargo test -p oxidoc-core --test search_golden` \
                     to write the golden files, then commit them";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Reads every `.bin` file in `dir` into a map from file name to bytes.
fn read_bin_files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bin"))
        .map(|path| {
            let name = path
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            (name, bytes)
        })
        .collect()
}

/// Builds the lexical index for the fixture site and returns its output files.
fn build_fixture_index() -> BTreeMap<String, Vec<u8>> {
    let fixture = manifest_dir().join("tests/fixtures/search-golden");
    let config = load_config(&fixture).expect("fixture oxidoc.toml must load");
    let nav_groups = discover_pages(&fixture, &config).expect("fixture pages must be found");
    let pages = extract_page_text(&nav_groups).expect("fixture pages must parse");
    assert!(!pages.is_empty(), "fixture has no pages");

    let out = tempfile::tempdir().expect("temp dir");
    write_lexical_index(&build_lexical_index(&pages), out.path())
        .expect("lexical index must write");
    read_bin_files(out.path())
}

/// Checks that the built files decode and match the manifest.
fn check_decodes(files: &BTreeMap<String, Vec<u8>>) {
    let meta_bytes = files
        .get("search-meta.bin")
        .expect("search-meta.bin must be written");
    let meta: SearchMetadata = rkyv::from_bytes::<_, rkyv::rancor::Error>(meta_bytes)
        .expect("search-meta.bin must decode");
    assert!(!meta.documents.is_empty());
    assert_eq!(files.len(), meta.manifest.chunks.len() + 1);

    for entry in &meta.manifest.chunks {
        let name = format!("search-chunk-{}.bin", entry.id);
        let bytes = files
            .get(&name)
            .unwrap_or_else(|| panic!("{name} is in the manifest but not written"));
        let chunk: ChunkPostings = rkyv::from_bytes::<_, rkyv::rancor::Error>(bytes)
            .unwrap_or_else(|e| panic!("{name} must decode: {e}"));
        assert!(!chunk.is_empty(), "{name} is empty");
    }
}

/// Replaces the golden `.bin` files with `built`.
fn write_goldens(golden: &Path, built: &BTreeMap<String, Vec<u8>>) {
    std::fs::create_dir_all(golden).expect("create golden dir");
    for name in read_bin_files(golden).keys() {
        std::fs::remove_file(golden.join(name)).expect("remove stale golden file");
    }
    for (name, bytes) in built {
        std::fs::write(golden.join(name), bytes).expect("write golden file");
    }
}

#[test]
fn lexical_index_matches_golden() {
    let built = build_fixture_index();
    check_decodes(&built);

    let golden = manifest_dir().join("tests/golden/search");
    if std::env::var_os(UPDATE_ENV).is_some_and(|v| v == "1") {
        write_goldens(&golden, &built);
        return;
    }

    assert!(
        golden.is_dir(),
        "golden directory {} is missing; {RERUN}",
        golden.display()
    );
    let expected = read_bin_files(&golden);

    for name in built.keys() {
        assert!(
            expected.contains_key(name),
            "golden file {}/{name} is missing; {RERUN}",
            golden.display()
        );
    }
    for name in expected.keys() {
        assert!(
            built.contains_key(name),
            "golden file {}/{name} is stale: the build no longer writes it; {RERUN}",
            golden.display()
        );
    }
    for (name, bytes) in &built {
        let want = &expected[name];
        assert!(
            want == bytes,
            "{name} differs from golden {}/{name} ({} bytes built, {} bytes golden); \
             if the change is intended, {RERUN}",
            golden.display(),
            bytes.len(),
            want.len()
        );
    }
}

#[test]
fn lexical_index_build_is_repeatable() {
    let first = build_fixture_index();
    let second = build_fixture_index();
    assert_eq!(
        first.keys().collect::<Vec<_>>(),
        second.keys().collect::<Vec<_>>()
    );
    for (name, bytes) in &first {
        assert!(second[name] == *bytes, "{name} differs between two builds");
    }
}
