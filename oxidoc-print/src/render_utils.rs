use oxipdf::ir::node::content::ImageFormat;
use oxipdf::ir::style::visual::{BorderSide, BorderStyle};
use oxipdf::ir::units::Pt;
use oxipdf::ir::{ResolvedStyle, SemanticRole};
use rdx_ast::Node;

use crate::config::PrintConfig;
use crate::error::{PrintError, Result};

/// Apply a thin colored border to a node for debug visualization.
/// Color indicates the semantic role of the element.
pub(crate) fn apply_debug_border(style: &mut ResolvedStyle, role: Option<SemanticRole>) {
    use oxipdf::ir::color::Color;

    let color = match role {
        Some(SemanticRole::Document) => Color::from_rgb8(200, 200, 200),
        Some(SemanticRole::Paragraph) => Color::from_rgb8(0, 150, 0),
        Some(SemanticRole::Heading { .. }) => Color::from_rgb8(200, 0, 0),
        Some(SemanticRole::List) => Color::from_rgb8(0, 0, 200),
        Some(SemanticRole::ListItem) => Color::from_rgb8(100, 100, 255),
        Some(SemanticRole::BlockQuote) => Color::from_rgb8(180, 100, 0),
        Some(SemanticRole::CodeBlock) => Color::from_rgb8(150, 0, 150),
        Some(SemanticRole::Table) => Color::from_rgb8(0, 150, 150),
        Some(SemanticRole::Figure) => Color::from_rgb8(200, 150, 0),
        Some(SemanticRole::Footnote) => Color::from_rgb8(255, 100, 100),
        _ => Color::from_rgb8(150, 150, 150),
    };

    let border = BorderSide {
        width: Pt::new(0.5),
        style: BorderStyle::Solid,
        color,
    };
    style.visual.border_top = border;
    style.visual.border_bottom = border;
    style.visual.border_left = border;
    style.visual.border_right = border;
}

/// Recursively collect all text content from a node tree.
pub(crate) fn collect_text_deep(node: &Node) -> String {
    match node {
        Node::Text(t) => t.value.clone(),
        Node::CodeInline(n) => n.value.clone(),
        _ => {
            if let Some(children) = node.children() {
                children.iter().map(collect_text_deep).collect()
            } else {
                String::new()
            }
        }
    }
}

/// Collect text from a flat slice of inline nodes.
pub(crate) fn collect_text(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            Node::Text(t) => out.push_str(&t.value),
            Node::Emphasis(n) | Node::Strong(n) | Node::Strikethrough(n) => {
                out.push_str(&collect_text(&n.children));
            }
            Node::CodeInline(n) => out.push_str(&n.value),
            Node::Link(n) => out.push_str(&collect_text(&n.children)),
            _ => {}
        }
    }
    out
}

/// Load an image file and determine its format from extension.
pub(crate) fn load_image(path: &std::path::Path) -> Result<(Vec<u8>, ImageFormat)> {
    let data = std::fs::read(path).map_err(|e| PrintError::FileRead {
        path: path.display().to_string(),
        source: e,
    })?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = match ext.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::Webp,
        _ => {
            return Err(PrintError::UnsupportedImageFormat {
                path: path.display().to_string(),
            });
        }
    };
    Ok((data, format))
}

/// Create a solid border with the given width and color.
pub(crate) fn solid_border(width: Pt, color: oxipdf::ir::color::Color) -> BorderSide {
    BorderSide {
        width,
        style: BorderStyle::Solid,
        color,
    }
}

impl PrintConfig {
    pub(crate) fn fonts(&self) -> &crate::config::FontConfig {
        &self.fonts
    }

    pub(crate) fn typo(&self) -> &crate::config::TypographyConfig {
        &self.typography
    }
}
