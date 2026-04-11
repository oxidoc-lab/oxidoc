use oxipdf::ir::node::content::{
    TableCell, TableColumn, TableColumnWidth, TableContent, TableRow, TableRowGroup,
    TableRowGroupKind,
};
use oxipdf::ir::style::layout::Display;
use oxipdf::ir::units::{Dimension, Pt};
use oxipdf::ir::{ContentVariant, NodeId, ResolvedStyle, SemanticRole};
use rdx_ast::{Node, StandardBlockNode};

use crate::render_utils::solid_border;
use crate::styles::InlineStyle;

use super::context::RenderContext;

impl<'a> RenderContext<'a> {
    pub(crate) fn render_table(&mut self, node: &StandardBlockNode, parent: NodeId) {
        let mut header_rows: Vec<&StandardBlockNode> = Vec::new();
        let mut body_rows: Vec<&StandardBlockNode> = Vec::new();
        let mut first = true;
        for child in &node.children {
            if let Node::TableRow(row) = child {
                if first {
                    header_rows.push(row);
                    first = false;
                } else {
                    body_rows.push(row);
                }
            }
        }

        let col_count = header_rows
            .first()
            .map(|r| {
                r.children
                    .iter()
                    .filter(|c| matches!(c, Node::TableCell(_)))
                    .count()
            })
            .unwrap_or(0);
        if col_count == 0 {
            return;
        }

        let columns: Vec<TableColumn> = (0..col_count)
            .map(|_| TableColumn {
                width: TableColumnWidth::Auto,
            })
            .collect();

        // Collect all rows with their header flag
        let all_rows: Vec<(&StandardBlockNode, bool)> = header_rows
            .iter()
            .map(|r| (*r, true))
            .chain(body_rows.iter().map(|r| (*r, false)))
            .collect();

        let total_cells: usize = all_rows
            .iter()
            .map(|(r, _)| {
                r.children
                    .iter()
                    .filter(|c| matches!(c, Node::TableCell(_)))
                    .count()
            })
            .sum();
        if total_cells == 0 {
            return;
        }

        // Predict cell IDs: table is at builder.len(), cells follow immediately
        let table_offset = self.builder.len() as u32;
        let first_cell = table_offset + 1;

        let mut idx: u32 = 0;
        let mut h_rows = Vec::new();
        let mut b_rows = Vec::new();
        for (row, is_hdr) in &all_rows {
            let mut cells = Vec::new();
            for child in &row.children {
                if matches!(child, Node::TableCell(_)) {
                    cells.push(TableCell {
                        content_node: NodeId::from_raw(first_cell + idx),
                        colspan: 1,
                        rowspan: 1,
                    });
                    idx += 1;
                }
            }
            if *is_hdr {
                h_rows.push(TableRow { cells });
            } else {
                b_rows.push(TableRow { cells });
            }
        }

        let mut groups = Vec::new();
        if !h_rows.is_empty() {
            groups.push(TableRowGroup {
                kind: TableRowGroupKind::Header,
                rows: h_rows,
            });
        }
        if !b_rows.is_empty() {
            groups.push(TableRowGroup {
                kind: TableRowGroupKind::Body,
                rows: b_rows,
            });
        }

        let tc = TableContent {
            columns,
            row_groups: groups,
            border_collapse: oxipdf::ir::node::content::BorderCollapse::Collapse,
            cell_spacing_h: Pt::ZERO,
            cell_spacing_v: Pt::ZERO,
            table_layout: oxipdf::ir::node::content::TableLayoutMode::Auto,
        };

        let mut ts = ResolvedStyle::default();
        ts.layout.display = Display::Block;
        ts.layout.margin_bottom = Dimension::Length(self.config.typography.paragraph_spacing);
        ts.visual.border_top = solid_border(Pt::new(1.0), oxipdf::ir::color::Color::BLACK);
        ts.visual.border_bottom = solid_border(Pt::new(1.0), oxipdf::ir::color::Color::BLACK);

        let table = self.builder.add_child(
            parent,
            ContentVariant::Table(tc),
            ts,
            Some(SemanticRole::Table),
            node.id.as_deref().map(String::from),
        );

        // Phase 1: add ALL cell containers as children of table
        let mut cell_ids = Vec::with_capacity(total_cells);
        for (row, is_hdr) in &all_rows {
            for child in &row.children {
                if matches!(child, Node::TableCell(_)) {
                    let cs = self.styles.table_cell(*is_hdr);
                    let cid =
                        self.builder
                            .add_child(table, ContentVariant::Container, cs, None, None);
                    cell_ids.push(cid);
                }
            }
        }

        // Phase 2: render inline content inside each cell
        let mut ci = 0;
        for (row, is_hdr) in &all_rows {
            for child in &row.children {
                if let Node::TableCell(cell_node) = child {
                    let inline = InlineStyle::new(self.config.fonts(), self.config.typo());
                    let inline = if *is_hdr { inline.with_bold() } else { inline };
                    for cc in &cell_node.children {
                        self.render_inline(cc, cell_ids[ci], &inline);
                    }
                    ci += 1;
                }
            }
        }
    }
}
