//! # oxidoc-print
//!
//! PDF rendering pipeline for the Oxidoc documentation engine.
//!
//! Converts RDX AST to publication-quality PDFs via the oxipdf rendering engine.
//! Supports configurable typography, system font discovery, and debug visualization.

pub mod config;
pub mod error;
pub mod fonts;
pub mod render_utils;
pub mod renderer;
pub mod styles;

use config::PrintConfig;
use error::Result;
use oxipdf::ir::config::RenderConfig;
use oxipdf::ir::page_template::{PageMargins, PageTemplate};

/// Render an RDX document to PDF bytes.
///
/// This is the primary entry point for the print pipeline:
/// 1. Converts the RDX AST to an oxipdf `StyledTree` (IR)
/// 2. Configures page layout (margins, size)
/// 3. Invokes oxipdf to produce PDF bytes
pub fn render_to_pdf(root: &rdx_ast::Root, config: &PrintConfig) -> Result<Vec<u8>> {
    // Step 1: Build the IR tree from the RDX AST
    let tree = renderer::render_to_tree(root, config)?;

    // Step 2: Configure the PDF render
    let page_width = config.page.size.width();
    let page_height = config.page.size.height();

    let margins = PageMargins {
        top: config.page.margin_top,
        bottom: config.page.margin_bottom,
        left: config.page.margin_inner,
        right: config.page.margin_outer,
    };

    let render_config = RenderConfig {
        page_width,
        page_height,
        page_template: Some(PageTemplate {
            margins,
            ..PageTemplate::default()
        }),
        hyphenation: oxipdf::ir::config::HyphenationConfig {
            enabled: true,
            min_word_length: 5,
            min_left: 2,
            min_right: 2,
        },
        ..RenderConfig::default()
    };

    // Step 3: Build font provider — load fonts matching configured families
    let families = [
        config.fonts.body.as_str(),
        config.fonts.heading.as_str(),
        config.fonts.mono.as_str(),
    ];
    let font_provider = fonts::build_font_provider_for_families(&config.fonts.font_dirs, &families);

    // Step 4: Render to PDF
    let pdf_bytes = match font_provider {
        Some(provider) => {
            tracing::info!(
                families = ?provider.available_families().len(),
                "Using shaped rendering with system fonts"
            );
            oxipdf::render_paginated_shaped_doc(&tree, &render_config, provider.as_ref())?
        }
        None => {
            tracing::warn!("No fonts found — using hardcoded metrics (output quality reduced)");
            oxipdf::render_paginated_doc(&tree, &render_config)?
        }
    };

    Ok(pdf_bytes)
}

/// Parse an RDX file and apply the print transform pipeline, then render to PDF.
///
/// This is a convenience function that combines parsing, transforms, and rendering.
pub fn render_file_to_pdf(rdx_source: &str, config: &PrintConfig) -> Result<Vec<u8>> {
    use rdx_transform::Transform;

    let mut root = rdx_parser::parse(rdx_source);

    // Apply shared transforms
    let auto_slug = rdx_transform::AutoSlug::new();
    let auto_number = rdx_transform::AutoNumber::new();

    auto_slug.transform(&mut root, rdx_source);
    auto_number.transform(&mut root, rdx_source);

    // Apply print-specific transforms
    let strip = rdx_transform::StripTarget {
        target: "print".into(),
    };
    let fallback = rdx_transform::PrintFallback;

    strip.transform(&mut root, rdx_source);
    fallback.transform(&mut root, rdx_source);

    // Resolve cross-references using the number registry from AutoNumber
    let registry = auto_number.registry();
    let cross_ref = rdx_transform::CrossRefResolve::new(
        rdx_transform::NumberRegistry {
            entries: registry.entries.clone(),
        },
        "print",
    );
    drop(registry);
    cross_ref.transform(&mut root, rdx_source);

    // Apply abbreviation expansion
    let abbrev = rdx_transform::AbbreviationExpand;
    abbrev.transform(&mut root, rdx_source);

    render_to_pdf(&root, config)
}
