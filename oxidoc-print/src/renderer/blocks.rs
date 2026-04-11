use oxipdf::ir::node::content::TextContent;
use oxipdf::ir::style::layout::Display;
use oxipdf::ir::style::typography::WhiteSpace;
use oxipdf::ir::units::Pt;
use oxipdf::ir::{ContentVariant, NodeId, ResolvedStyle, SemanticRole};
use rdx_ast::{CodeBlockNode, ComponentNode, ErrorNode, MathDisplayNode, Node, StandardBlockNode};

use crate::render_utils::{collect_text, collect_text_deep};
use crate::styles::InlineStyle;

use super::context::{ListState, RenderContext};

impl<'a> RenderContext<'a> {
    pub(crate) fn render_block(&mut self, node: &Node, parent: NodeId) {
        match node {
            Node::Paragraph(n) => self.render_paragraph(n, parent),
            Node::Heading(n) => self.render_heading(n, parent),
            Node::List(n) => self.render_list(n, parent),
            Node::ListItem(n) => self.render_list_item(n, parent),
            Node::Blockquote(n) => self.render_blockquote(n, parent),
            Node::ThematicBreak(_) => self.render_thematic_break(parent),
            Node::CodeBlock(n) => self.render_code_block(n, parent),
            Node::Table(n) => self.render_table(n, parent),
            Node::DefinitionList(n) => self.render_definition_list(n, parent),
            Node::DefinitionTerm(n) => self.render_definition_term(n, parent),
            Node::DefinitionDescription(n) => self.render_definition_desc(n, parent),
            Node::MathDisplay(n) => self.render_math_display(n, parent),
            // Footnote definitions are consumed in pre-pass and attached at reference sites.
            // Do NOT render them as block content — they'd appear twice.
            Node::FootnoteDefinition(_) => {}
            Node::Html(n) => {
                let para = self.add_container(
                    parent,
                    self.styles.paragraph(),
                    SemanticRole::Paragraph,
                    None,
                );
                let text = collect_text(&n.children);
                if !text.is_empty() {
                    let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
                    self.add_text(&text, para, &inline);
                }
            }
            Node::Component(n) => self.render_component(n, parent),
            Node::Error(n) => self.render_error(n, parent),
            // Inline nodes at block level — wrap in implicit paragraph
            Node::Text(_)
            | Node::Emphasis(_)
            | Node::Strong(_)
            | Node::Strikethrough(_)
            | Node::CodeInline(_)
            | Node::Link(_)
            | Node::Image(_)
            | Node::MathInline(_)
            | Node::Citation(_)
            | Node::CrossRef(_)
            | Node::Variable(_)
            | Node::FootnoteReference(_) => {
                let para = self.add_container(
                    parent,
                    self.styles.paragraph(),
                    SemanticRole::Paragraph,
                    None,
                );
                let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
                self.render_inline(node, para, &inline);
            }
            Node::TableRow(_) | Node::TableCell(_) => {}
        }
    }

    fn render_paragraph(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let para = self.add_container(
            parent,
            self.styles.paragraph(),
            SemanticRole::Paragraph,
            node.id.as_deref(),
        );
        let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
        for child in &node.children {
            self.render_inline(child, para, &inline);
        }
    }

    fn render_heading(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let level = node.depth.unwrap_or(1);
        let heading = self.add_container(
            parent,
            self.styles.heading(level),
            SemanticRole::Heading { level },
            node.id.as_deref(),
        );
        let mut inline = InlineStyle::new(self.config.fonts(), self.config.typo());
        inline.font_families = vec![self.config.fonts.heading.clone()];
        inline.font_weight = 700;
        inline.font_size =
            self.config.typography.heading_sizes[(level.saturating_sub(1) as usize).min(5)];
        for child in &node.children {
            self.render_inline(child, heading, &inline);
        }
    }

    fn render_list(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let ordered = node.ordered.unwrap_or(false);
        let list = self.add_container(parent, self.styles.list(), SemanticRole::List, None);
        self.list_counter.push(ListState { ordered, index: 0 });
        for child in &node.children {
            self.render_block(child, list);
        }
        self.list_counter.pop();
    }

    fn render_list_item(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let (ordered, index) = if let Some(state) = self.list_counter.last_mut() {
            state.index += 1;
            (state.ordered, state.index)
        } else {
            (false, 1)
        };

        let item = self.add_container(
            parent,
            self.styles.list_item(ordered, index),
            SemanticRole::ListItem,
            None,
        );

        // Workaround: oxipdf stacks multiple inline Text children vertically
        // instead of flowing them on one line for single-line containers.
        // Concatenate all text into one Text node. Loses inline formatting
        // (bold/italic inside list items) but gives correct compact spacing.
        let marker: String = if ordered {
            format!("{}. ", index)
        } else {
            "\u{2022} ".into()
        };
        let mut full_text = marker;
        for child in &node.children {
            full_text.push_str(&collect_text_deep(child));
        }
        let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
        self.add_text(&full_text, item, &inline);
    }

    fn render_blockquote(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let bq = self.add_container(
            parent,
            self.styles.blockquote(),
            SemanticRole::BlockQuote,
            None,
        );
        for child in &node.children {
            self.render_block(child, bq);
        }
    }

    fn render_thematic_break(&mut self, parent: NodeId) {
        self.builder.add_child(
            parent,
            ContentVariant::Container,
            self.styles.thematic_break(),
            None,
            None,
        );
    }

    fn render_code_block(&mut self, node: &CodeBlockNode, parent: NodeId) {
        // Optional title bar
        if let Some(ref title) = node.title {
            let title_container = self.builder.add_child(
                parent,
                ContentVariant::Container,
                self.styles.code_title(),
                None,
                None,
            );
            let mut ts = ResolvedStyle::default();
            ts.layout.display = Display::Inline;
            ts.typography.font_families = vec![self.config.fonts.mono.clone()];
            ts.typography.font_size = Pt::new(self.config.typography.code_size.get() * 0.9);
            ts.typography.font_weight = 600;
            ts.typography.color = oxipdf::ir::color::Color::from_rgb8(88, 96, 105);
            self.builder.add_child(
                title_container,
                ContentVariant::Text(TextContent::new(title)),
                ts,
                None,
                None,
            );
        }

        let code_container = self.add_container(
            parent,
            self.styles.code_block(),
            SemanticRole::CodeBlock,
            None,
        );

        let lang = node.lang.as_deref().unwrap_or("");
        let code = &node.value;
        let highlight_lines = node.highlight.as_deref().unwrap_or(&[]);
        let show_ln = node.show_line_numbers.unwrap_or(false);
        let is_diff = node.diff.unwrap_or(false);

        if !lang.is_empty() && oxidoc_highlight::is_supported(lang) {
            self.render_code_highlighted(
                code,
                lang,
                code_container,
                highlight_lines,
                show_ln,
                is_diff,
            );
        } else {
            self.render_code_plain(code, code_container, highlight_lines, show_ln, is_diff);
        }
    }

    /// Render code — one line per paragraph-like container.
    /// Each line is a Container(Block) with one Text(Inline) child.
    /// This produces correct text rendering (single-child inline containers
    /// work correctly in oxipdf's shaping path).
    fn render_code_plain(
        &mut self,
        code: &str,
        parent: NodeId,
        _highlight_lines: &[u32],
        show_ln: bool,
        _is_diff: bool,
    ) {
        for (i, line) in code.lines().enumerate() {
            let mut text = String::new();
            if show_ln {
                text.push_str(&format!("{:>4}  ", i + 1));
            }
            text.push_str(line);

            // Paragraph-like container with mono typography
            let mut para_style = ResolvedStyle::default();
            para_style.layout.display = Display::Block;
            para_style.typography.font_families = vec![self.config.fonts.mono.clone()];
            para_style.typography.font_size = self.config.typography.code_size;
            para_style.typography.line_height = oxipdf::ir::style::typography::LineHeight::Number(
                self.config.typography.code_line_height,
            );
            para_style.typography.white_space = WhiteSpace::Pre;
            let line_container =
                self.builder
                    .add_child(parent, ContentVariant::Container, para_style, None, None);

            // Single inline text child
            let mut ts = ResolvedStyle::default();
            ts.layout.display = Display::Inline;
            ts.typography.font_families = vec![self.config.fonts.mono.clone()];
            ts.typography.font_size = self.config.typography.code_size;
            ts.typography.white_space = WhiteSpace::Pre;
            ts.typography.color = oxipdf::ir::color::Color::from_rgb8(36, 41, 46);
            self.builder.add_child(
                line_container,
                ContentVariant::Text(TextContent::new(&text)),
                ts,
                None,
                None,
            );
        }
    }

    /// Render code with syntax highlighting — same as plain for now.
    fn render_code_highlighted(
        &mut self,
        code: &str,
        _lang: &str,
        parent: NodeId,
        highlight_lines: &[u32],
        show_ln: bool,
        is_diff: bool,
    ) {
        self.render_code_plain(code, parent, highlight_lines, show_ln, is_diff);
    }

    fn render_definition_list(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let dl = self.add_container(parent, self.styles.list(), SemanticRole::List, None);
        for child in &node.children {
            self.render_block(child, dl);
        }
    }

    fn render_definition_term(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let dt = self.builder.add_child(
            parent,
            ContentVariant::Container,
            self.styles.definition_term(),
            None,
            None,
        );
        let mut inline = InlineStyle::new(self.config.fonts(), self.config.typo());
        inline.font_weight = 700;
        for child in &node.children {
            self.render_inline(child, dt, &inline);
        }
    }

    fn render_definition_desc(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let dd = self.builder.add_child(
            parent,
            ContentVariant::Container,
            self.styles.definition_description(),
            None,
            None,
        );
        for child in &node.children {
            match child {
                Node::Paragraph(_) | Node::List(_) | Node::CodeBlock(_) => {
                    self.render_block(child, dd)
                }
                _ => {
                    let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
                    self.render_inline(child, dd, &inline);
                }
            }
        }
    }

    fn render_math_display(&mut self, node: &MathDisplayNode, parent: NodeId) {
        // Render as centered monospace text (proper MATH font rendering is a later enhancement)
        let container = self.add_container(
            parent,
            self.styles.paragraph(),
            SemanticRole::Paragraph,
            node.label.as_deref(),
        );
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Inline;
        s.typography.font_families = vec![self.config.fonts.mono.clone()];
        s.typography.font_size = self.config.typography.body_size;
        s.typography.color = oxipdf::ir::color::Color::BLACK;
        self.builder.add_child(
            container,
            ContentVariant::Text(TextContent::new(&node.raw)),
            s,
            None,
            None,
        );
    }

    fn render_component(&mut self, node: &ComponentNode, parent: NodeId) {
        match node.name.as_str() {
            "PageBreak" => {
                let mut s = ResolvedStyle::default();
                s.layout.display = Display::Block;
                s.fragmentation.break_before = oxipdf::ir::style::fragmentation::BreakValue::Always;
                self.builder
                    .add_child(parent, ContentVariant::Container, s, None, None);
            }
            "WebOnly" => {} // stripped by StripTarget
            "PrintOnly" => {
                for child in &node.children {
                    self.render_block(child, parent);
                }
            }
            _ => {
                tracing::debug!(component = %node.name, "Unhandled component — rendering children");
                for child in &node.children {
                    self.render_block(child, parent);
                }
            }
        }
    }

    fn render_error(&mut self, node: &ErrorNode, parent: NodeId) {
        let err = self.builder.add_child(
            parent,
            ContentVariant::Container,
            self.styles.error(),
            None,
            None,
        );
        let mut s = self.styles.error();
        s.layout.display = Display::Inline;
        self.builder.add_child(
            err,
            ContentVariant::Text(TextContent::new(format!(
                "Error: {} — {}",
                node.message, node.raw_content
            ))),
            s,
            None,
            None,
        );
    }
}
