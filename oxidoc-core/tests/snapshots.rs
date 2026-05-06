//! Integration tests for HTML output stability and structural validation.
//!
//! These tests ensure that the build process produces well-formed HTML
//! with expected structural elements and components.

use oxidoc_core::builder::build_site;

/// Helper to create a temporary test project with config and files.
fn setup_project(
    config_toml: &str,
    files: &[(&str, &str)],
) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = tmp.path();
    let docs = root.join("docs");
    std::fs::create_dir(&docs).expect("Failed to create docs dir");
    std::fs::write(root.join("oxidoc.toml"), config_toml).expect("Failed to write config");
    for (name, content) in files {
        std::fs::write(docs.join(name), content).expect("Failed to write file");
    }
    let output = root.join("dist");
    (tmp, output)
}

#[test]
fn snapshot_basic_page() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Test Docs\"\n",
        &[("intro.rdx", "# Introduction\n\nWelcome to our docs!")],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    let html = std::fs::read_to_string(output.join("intro.html"))
        .expect("Failed to read generated HTML");

    // Assert structural elements
    assert!(html.contains("<!DOCTYPE html>"), "Missing DOCTYPE");
    assert!(html.contains("<html"), "Missing html tag");
    assert!(html.contains("<head>"), "Missing head tag");
    assert!(
        html.contains(r#"<body data-locale=""#),
        "Missing body tag with locale"
    );
    assert!(html.contains("</html>"), "Missing closing html tag");

    // Assert page structure
    assert!(html.contains("<header"), "Missing header");
    assert!(html.contains("<main"), "Missing main content");
    assert!(html.contains("<article"), "Missing article");
    assert!(html.contains("</article>"), "Missing closing article tag");
    assert!(html.contains("</main>"), "Missing closing main tag");

    // Assert title
    assert!(
        html.contains("Test Docs"),
        "Page should reference site name"
    );

    // Assert content - check that key text is present (rendering may vary)
    assert!(
        html.contains("Introduction") || html.contains("introduction"),
        "Page should contain the intro text"
    );
    assert!(html.contains("Welcome"), "Page should contain welcome text");

    // Assert all opened tags have closing pairs (basic validation)
    assert_eq!(html.matches("<html").count(), 1);
    assert_eq!(html.matches("</html>").count(), 1);
    assert_eq!(html.matches("<head>").count(), 1);
    assert_eq!(html.matches("</head>").count(), 1);
    assert_eq!(html.matches("<body").count(), 1);
    assert_eq!(html.matches("</body>").count(), 1);
}

#[test]
fn snapshot_page_with_metadata() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"API Docs\"\ndescription = \"Complete API reference\"\n",
        &[("api.rdx", "# API Reference\n\nComplete API documentation.")],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    let html = std::fs::read_to_string(output.join("api.html"))
        .expect("Failed to read generated HTML");

    // Assert SEO metadata
    assert!(html.contains(r#"<meta name="description""#));
    assert!(html.contains("API"), "Page should mention API");
    assert!(html.contains(r#"<meta name="generator" content="oxidoc""#));
    assert!(
        html.contains(r#"<meta property="og:title""#),
        "Missing Open Graph title"
    );
    assert!(
        html.contains(r#"<meta property="og:type" content="article""#),
        "Missing Open Graph type"
    );
    assert!(
        html.contains(r#"<meta property="og:description""#),
        "Missing Open Graph description"
    );
    assert!(
        html.contains(r#"<meta name="twitter:card""#),
        "Missing Twitter card"
    );
}

#[test]
fn snapshot_404_page() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Test\"\n",
        &[("home.rdx", "# Home\n\nWelcome!")],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    let not_found_html =
        std::fs::read_to_string(output.join("404.html")).expect("Failed to read 404 page");

    assert!(
        not_found_html.contains("<!DOCTYPE html>"),
        "404 missing DOCTYPE"
    );
    assert!(not_found_html.contains("<h1>404"), "404 missing heading");
    assert!(
        not_found_html.contains("Not Found"),
        "404 missing standard message"
    );
    assert!(
        not_found_html.contains("Return to home"),
        "404 missing help link"
    );
    assert!(
        not_found_html.contains("<header"),
        "404 missing header navigation"
    );
    assert!(not_found_html.contains("<main"), "404 missing main content");
}

#[test]
fn snapshot_page_with_sections() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Guide\"\n",
        &[(
            "tutorial.rdx",
            "# Tutorial\n\nIntroduction.\n\n## Getting Started\n\nFirst steps.\n\n## Advanced Usage\n\nMore info.",
        )],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    let html = std::fs::read_to_string(output.join("tutorial.html"))
        .expect("Failed to read generated HTML");

    // Verify key content is present (h1/h2 rendering may vary)
    assert!(
        html.contains("Tutorial") || html.contains("tutorial"),
        "Should contain tutorial text"
    );
    assert!(
        html.contains("Getting Started") || html.contains("getting started"),
        "Should contain section text"
    );
    assert!(
        html.contains("Advanced Usage") || html.contains("advanced usage"),
        "Should contain advanced section"
    );

    // Verify paragraph content
    assert!(html.contains("Introduction") || html.contains("introduction"));
    assert!(
        html.contains("First steps") || html.contains("first steps"),
        "Should contain setup text"
    );
    assert!(html.contains("More info") || html.contains("more info"));
}

#[test]
fn snapshot_assets_are_hashed() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Hashed\"\n",
        &[("page.rdx", "# Page\n\nContent.")],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    // Find the hashed CSS file
    let css_files: Vec<_> = std::fs::read_dir(&output)
        .expect("Failed to read output dir")
        .filter_map(|e| {
            let p = e.ok()?.path();
            let name = p.file_name()?.to_string_lossy().to_string();
            if name.starts_with("oxidoc.") && name.ends_with(".css") {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        css_files.len(),
        1,
        "Should have exactly one hashed CSS file"
    );

    // Find the hashed JS file
    let js_files: Vec<_> = std::fs::read_dir(&output)
        .expect("Failed to read output dir")
        .filter_map(|e| {
            let p = e.ok()?.path();
            let name = p.file_name()?.to_string_lossy().to_string();
            if name.starts_with("oxidoc-loader.") && name.ends_with(".js") {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(js_files.len(), 1, "Should have exactly one hashed JS file");

    // Verify the HTML references the hashed assets
    let html = std::fs::read_to_string(output.join("page.html"))
        .expect("Failed to read HTML");

    let css_name = css_files[0].file_name().unwrap().to_string_lossy();
    let js_name = js_files[0].file_name().unwrap().to_string_lossy();

    assert!(
        html.contains(&format!(r#"href="/{}"#, css_name)),
        "HTML should reference hashed CSS"
    );
    assert!(
        html.contains(&format!(r#"src="/{}"#, js_name)),
        "HTML should reference hashed JS"
    );
}

#[test]
fn snapshot_sri_hashes_present() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"SRI Test\"\n",
        &[("secure.rdx", "# Secure\n\nWith integrity checks.")],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    let html = std::fs::read_to_string(output.join("secure.html"))
        .expect("Failed to read HTML");

    // Check for SRI attributes
    assert!(
        html.contains(r#"integrity="sha384-"#),
        "Should have integrity attribute for CSS"
    );
    assert!(
        html.contains(r#"crossorigin="anonymous""#),
        "Should have crossorigin attribute for SRI"
    );
}

/// Helper to build a root-page project (routing.root.homepage).
fn setup_root_project(
    config_toml: &str,
    homepage_file: &str,
    homepage_content: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = tmp.path();
    let docs = root.join("docs");
    std::fs::create_dir(&docs).expect("Failed to create docs dir");
    std::fs::write(root.join("oxidoc.toml"), config_toml).expect("Failed to write config");
    std::fs::write(root.join(homepage_file), homepage_content).expect("Failed to write homepage");
    let output = root.join("dist");
    (tmp, output)
}

#[test]
fn frontmatter_title_used_on_landing_page() {
    let config = "[project]\nname = \"NodeDB\"\n\n[routing.root]\nhomepage = \"home.rdx\"\n";
    let content =
        "---\ntitle: Hybrid Database for AI Workloads\nlayout: landing\n---\n\n<p>Welcome</p>";
    let (tmp, output) = setup_root_project(config, "home.rdx", content);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("index.html")).expect("index.html missing");
    assert!(
        html.contains("Hybrid Database for AI Workloads"),
        "frontmatter title should appear"
    );
    assert!(
        !html.contains("NodeDB - NodeDB"),
        "duplicated project name must not appear"
    );
}

#[test]
fn frontmatter_title_used_on_regular_page() {
    let config = "[project]\nname = \"Docs\"\n";
    let content = "---\ntitle: Getting Started\n---\n\n# Different Heading\n\nWelcome!";
    let (tmp, output) = setup_project(config, &[("start.rdx", content)]);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("start.html"))
        .expect("start.html missing");
    assert!(
        html.contains("Getting Started - Docs"),
        "should use frontmatter title"
    );
    assert!(
        !html.contains("Different Heading - Docs"),
        "h1 should not override frontmatter title"
    );
}

#[test]
fn frontmatter_description_used() {
    let config = "[project]\nname = \"Docs\"\n";
    let content = "---\ndescription: Frontmatter description here.\n---\n\nFirst paragraph text.";
    let (tmp, output) = setup_project(config, &[("page.rdx", content)]);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("page.html"))
        .expect("page.html missing");
    assert!(
        html.contains("Frontmatter description here."),
        "should use frontmatter description"
    );
    // The meta description should use frontmatter, not the first paragraph text
    assert!(
        html.contains(r#"content="Frontmatter description here.""#),
        "meta description should be the frontmatter description"
    );
    // The meta description must not contain the first paragraph text
    assert!(
        !html.contains(r#"content="First paragraph text"#),
        "meta description must not use first paragraph when frontmatter description is set"
    );
}

#[test]
fn homepage_emits_og_type_website() {
    let config = "[project]\nname = \"Site\"\n\n[routing.root]\nhomepage = \"home.rdx\"\n";
    let content = "---\ntitle: Home\nlayout: landing\n---\n\n<p>Welcome</p>";
    let (tmp, output) = setup_root_project(config, "home.rdx", content);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("index.html")).expect("index.html missing");
    assert!(
        html.contains(r#"og:type" content="website""#),
        "homepage must have og:type=website"
    );
    assert!(
        !html.contains(r#"og:type" content="article""#),
        "homepage must not have og:type=article"
    );
}

#[test]
fn regular_page_emits_og_type_article() {
    let config = "[project]\nname = \"Docs\"\n";
    let (tmp, output) = setup_project(config, &[("guide.rdx", "# Guide\n\nContent.")]);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("guide.html"))
        .expect("guide.html missing");
    assert!(html.contains(r#"og:type" content="article""#));
}

#[test]
fn head_block_override_no_duplicate_meta() {
    // A page using <Head> to set og:type should produce exactly one og:type tag
    // This requires oxidoc-components <Head> island support — if the component
    // doesn't exist yet this test documents the expected behavior.
    // For now, simulate by checking that remove_overridden_meta_tags works
    // at the integration level by using the builder with a page that includes
    // a Head override marker in its rendered content.
    // (Skipped if <Head> component isn't wired in renderer — checked at runtime)
    let config = "[project]\nname = \"Docs\"\n";
    let content = "# Page\n\nContent.<!--oxidoc-head-start--><meta property=\"og:type\" content=\"website\"><!--oxidoc-head-end-->";
    let (tmp, output) = setup_project(config, &[("page.rdx", content)]);
    build_site(tmp.path(), &output).expect("Build failed");
    let html = std::fs::read_to_string(output.join("page.html"))
        .expect("page.html missing");
    // Count real <meta> tags inside <head> only. The Copy Markdown dropdown
    // embeds the raw RDX source in a <script type="text/markdown"> block in
    // the body, which contains its own literal "og:type" substring that
    // should not be counted here.
    let head = html
        .split_once("</head>")
        .map(|(h, _)| h)
        .expect("missing </head>");
    assert_eq!(
        head.matches(r#"<meta property="og:type""#).count(),
        1,
        "exactly one og:type meta tag in <head> when Head block overrides it"
    );
}

#[test]
fn snapshot_multiple_pages() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Multi-page\"\n",
        &[
            ("home.rdx", "# Home\n\nWelcome!"),
            ("guide.rdx", "# Guide\n\nHow to use."),
            ("reference.rdx", "# Reference\n\nAPI docs."),
        ],
    );

    build_site(tmp.path(), &output).expect("Build failed");

    // Verify all pages are generated as directory output
    assert!(output.join("home.html").exists());
    assert!(output.join("guide.html").exists());
    assert!(output.join("reference.html").exists());

    // Verify each page is valid HTML
    let home = std::fs::read_to_string(output.join("home.html")).unwrap();
    assert!(
        home.contains("<!DOCTYPE html>"),
        "Home page should be valid HTML"
    );

    let guide = std::fs::read_to_string(output.join("guide.html")).unwrap();
    assert!(
        guide.contains("<!DOCTYPE html>"),
        "Guide page should be valid HTML"
    );

    let reference = std::fs::read_to_string(output.join("reference.html")).unwrap();
    assert!(
        reference.contains("<!DOCTYPE html>"),
        "Reference page should be valid HTML"
    );
}

#[test]
fn sitemap_uses_clean_urls() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Test\"\nbase_url = \"https://example.com\"\n",
        &[("page.rdx", "# Page\n\nContent.")],
    );
    build_site(tmp.path(), &output).expect("Build failed");
    let sitemap = std::fs::read_to_string(output.join("sitemap.xml")).expect("sitemap.xml missing");
    assert!(
        sitemap.contains("https://example.com/page"),
        "sitemap should use clean URL"
    );
    assert!(
        !sitemap.contains(".html"),
        "sitemap must not contain .html extensions"
    );
}
