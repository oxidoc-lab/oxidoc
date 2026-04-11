use oxipdf::ir::{
    ContentVariant, IrVersion, NodeId, ResolvedStyle, SemanticRole, StyledTree, StyledTreeBuilder,
};
use rdx_ast::{FootnoteNode, Node};

use crate::config::PrintConfig;
use crate::error::Result;
use crate::render_utils::apply_debug_border;
use crate::styles::{InlineStyle, StyleFactory};

use std::collections::HashMap;

/// Convert an RDX AST into an oxipdf `StyledTree`.
pub fn render_to_tree(root: &rdx_ast::Root, config: &PrintConfig) -> Result<StyledTree> {
    let styles = StyleFactory::new(&config.fonts, &config.typography);

    // Pre-pass: collect footnote definitions so we can attach them at reference sites
    let mut footnote_defs: HashMap<String, &FootnoteNode> = HashMap::new();
    collect_footnote_defs(&root.children, &mut footnote_defs);

    let mut ctx = RenderContext {
        builder: StyledTreeBuilder::new(IrVersion::new(1, 0)),
        styles: &styles,
        config,
        list_counter: Vec::new(),
        footnote_defs,
    };

    let root_id = ctx.builder.add_node(
        ContentVariant::Container,
        ctx.styles.document(),
        Some(SemanticRole::Document),
        None,
    );

    for child in &root.children {
        ctx.render_block(child, root_id);
    }

    let tree = ctx.builder.build()?;
    Ok(tree)
}

fn collect_footnote_defs<'a>(nodes: &'a [Node], map: &mut HashMap<String, &'a FootnoteNode>) {
    for node in nodes {
        if let Node::FootnoteDefinition(f) = node {
            map.insert(f.label.clone(), f);
        }
        if let Some(children) = node.children() {
            collect_footnote_defs(children, map);
        }
    }
}

pub(crate) struct RenderContext<'a> {
    pub(crate) builder: StyledTreeBuilder,
    pub(crate) styles: &'a StyleFactory<'a>,
    pub(crate) config: &'a PrintConfig,
    pub(crate) list_counter: Vec<ListState>,
    #[allow(dead_code)]
    pub(crate) footnote_defs: HashMap<String, &'a FootnoteNode>,
}

pub(crate) struct ListState {
    pub(crate) ordered: bool,
    pub(crate) index: u32,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn add_text(&mut self, text: &str, parent: NodeId, inherited: &InlineStyle) {
        let style = self.styles.inline_text(inherited);
        self.builder.add_child(
            parent,
            ContentVariant::Text(oxipdf::ir::node::content::TextContent::new(text)),
            style,
            None,
            None,
        );
    }

    pub(crate) fn add_container(
        &mut self,
        parent: NodeId,
        style: ResolvedStyle,
        role: SemanticRole,
        element_id: Option<&str>,
    ) -> NodeId {
        let mut style = style;
        if self.config.debug_boxes {
            apply_debug_border(&mut style, Some(role));
        }
        self.builder.add_child(
            parent,
            ContentVariant::Container,
            style,
            Some(role),
            element_id.map(String::from),
        )
    }
}
