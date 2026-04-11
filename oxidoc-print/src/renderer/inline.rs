use oxipdf::ir::node::content::{ImageContent, TextContent};
use oxipdf::ir::{ContentVariant, NodeId};
use rdx_ast::{
    CitationNode, CrossRefNode, FootnoteNode, ImageNode, LinkNode, MathNode, Node, TextNode,
    VariableNode,
};

use crate::render_utils::load_image;
use crate::styles::InlineStyle;

use super::context::RenderContext;

impl<'a> RenderContext<'a> {
    pub(crate) fn render_inline(&mut self, node: &Node, parent: NodeId, inherited: &InlineStyle) {
        match node {
            Node::Text(n) => self.render_text(n, parent, inherited),
            Node::Emphasis(n) => {
                let s = inherited.with_italic();
                for c in &n.children {
                    self.render_inline(c, parent, &s);
                }
            }
            Node::Strong(n) => {
                let s = inherited.with_bold();
                for c in &n.children {
                    self.render_inline(c, parent, &s);
                }
            }
            Node::Strikethrough(n) => {
                let s = inherited.with_strikethrough();
                for c in &n.children {
                    self.render_inline(c, parent, &s);
                }
            }
            Node::CodeInline(n) => {
                self.builder.add_child(
                    parent,
                    ContentVariant::Text(TextContent::new(&n.value)),
                    self.styles.inline_code(),
                    None,
                    None,
                );
            }
            Node::Link(n) => self.render_link(n, parent, inherited),
            Node::Image(n) => self.render_image(n, parent),
            Node::MathInline(n) => self.render_math_inline(n, parent, inherited),
            Node::Citation(n) => self.render_citation(n, parent, inherited),
            Node::CrossRef(n) => self.render_crossref(n, parent, inherited),
            Node::Variable(n) => self.render_variable(n, parent, inherited),
            Node::FootnoteReference(n) => self.render_footnote_ref(n, parent, inherited),
            _ => self.render_block(node, parent),
        }
    }

    fn render_text(&mut self, node: &TextNode, parent: NodeId, inherited: &InlineStyle) {
        if node.value.is_empty() {
            return;
        }
        self.add_text(&node.value, parent, inherited);
    }

    fn render_link(&mut self, node: &LinkNode, parent: NodeId, inherited: &InlineStyle) {
        // Render link text inline (ContentVariant::Link breaks paragraph flow because
        // oxipdf's inline detection doesn't handle Link nodes). Clickable annotations
        // will be added when oxipdf supports inline Link nodes properly.
        let link_style = inherited
            .with_color(oxipdf::ir::color::Color::from_rgb8(3, 102, 214))
            .with_underline();
        for child in &node.children {
            self.render_inline(child, parent, &link_style);
        }
    }

    fn render_image(&mut self, node: &ImageNode, parent: NodeId) {
        let path = self.config.project_root.join(&node.url);
        match load_image(&path) {
            Ok((data, format)) => {
                let alt = node.alt.as_deref().or(Some(&node.url));
                let mut img = ImageContent::new(data, format);
                if let Some(a) = alt {
                    img = img.with_alt_text(a.to_string());
                }
                self.builder.add_child(
                    parent,
                    ContentVariant::Image(img),
                    self.styles.image(),
                    None,
                    None,
                );
            }
            Err(e) => {
                tracing::warn!("Failed to load image {}: {e}", node.url);
                let alt = node.alt.as_deref().unwrap_or(&node.url);
                let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
                self.add_text(&format!("[Image: {alt}]"), parent, &inline);
            }
        }
    }

    fn render_math_inline(&mut self, node: &MathNode, parent: NodeId, inherited: &InlineStyle) {
        // Render as monospace text (proper MATH font rendering is a later enhancement)
        let mono = inherited.with_mono(self.config.fonts(), self.config.typo());
        self.add_text(&node.raw, parent, &mono);
    }

    fn render_citation(&mut self, node: &CitationNode, parent: NodeId, inherited: &InlineStyle) {
        let text = node
            .keys
            .iter()
            .map(|k| format!("@{}", k.id))
            .collect::<Vec<_>>()
            .join("; ");
        self.add_text(&format!("[{text}]"), parent, inherited);
    }

    fn render_crossref(&mut self, node: &CrossRefNode, parent: NodeId, inherited: &InlineStyle) {
        self.add_text(&format!("[ref: {}]", node.target), parent, inherited);
    }

    fn render_variable(&mut self, node: &VariableNode, parent: NodeId, inherited: &InlineStyle) {
        self.add_text(&format!("{{{}}}", node.path), parent, inherited);
    }

    fn render_footnote_ref(
        &mut self,
        node: &FootnoteNode,
        parent: NodeId,
        inherited: &InlineStyle,
    ) {
        // Render superscript marker only. Footnote body placement at page bottom
        // requires oxipdf to properly suppress Footnote children in-flow, which is
        // not yet working. For now, just show the marker.
        let sup = inherited.with_superscript();
        self.add_text(&node.label, parent, &sup);
    }
}
