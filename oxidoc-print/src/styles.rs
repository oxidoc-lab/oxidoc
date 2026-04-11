use oxipdf::ir::ResolvedStyle;
use oxipdf::ir::color::Color;
use oxipdf::ir::style::layout::Display;
use oxipdf::ir::style::list::{ListMarker, ListMarkerPosition};
use oxipdf::ir::style::typography::{
    FontStyle, LineHeight, TextAlign, TextDecoration, VerticalAlign, WhiteSpace,
};
use oxipdf::ir::style::visual::{BorderSide, BorderStyle};
use oxipdf::ir::units::{Dimension, LengthPercentage, Pt};

use crate::config::{FontConfig, TypographyConfig};

/// Inline style state propagated through inline content (emphasis, strong, etc.).
#[derive(Clone)]
pub struct InlineStyle {
    pub font_weight: u16,
    pub font_style: FontStyle,
    pub text_decoration: TextDecoration,
    pub color: Color,
    pub font_families: Vec<String>,
    pub font_size: Pt,
    pub is_superscript: bool,
}

impl InlineStyle {
    pub fn new(fonts: &FontConfig, typo: &TypographyConfig) -> Self {
        Self {
            font_weight: 400,
            font_style: FontStyle::Normal,
            text_decoration: TextDecoration::None,
            color: Color::BLACK,
            font_families: vec![fonts.body.clone()],
            font_size: typo.body_size,
            is_superscript: false,
        }
    }

    pub fn with_bold(&self) -> Self {
        let mut s = self.clone();
        s.font_weight = 700;
        s
    }

    pub fn with_italic(&self) -> Self {
        let mut s = self.clone();
        s.font_style = FontStyle::Italic;
        s
    }

    pub fn with_strikethrough(&self) -> Self {
        let mut s = self.clone();
        s.text_decoration = TextDecoration::LineThrough;
        s
    }

    pub fn with_underline(&self) -> Self {
        let mut s = self.clone();
        s.text_decoration = TextDecoration::Underline;
        s
    }

    pub fn with_color(&self, color: Color) -> Self {
        let mut s = self.clone();
        s.color = color;
        s
    }

    pub fn with_mono(&self, fonts: &FontConfig, typo: &TypographyConfig) -> Self {
        let mut s = self.clone();
        s.font_families = vec![fonts.mono.clone()];
        s.font_size = typo.code_size;
        s
    }

    pub fn with_superscript(&self) -> Self {
        let mut s = self.clone();
        s.is_superscript = true;
        s.font_size = Pt::new(s.font_size.get() * 0.7);
        s
    }
}

/// Style factory — builds `ResolvedStyle` for each element type.
pub struct StyleFactory<'a> {
    pub fonts: &'a FontConfig,
    pub typo: &'a TypographyConfig,
}

impl<'a> StyleFactory<'a> {
    pub fn new(fonts: &'a FontConfig, typo: &'a TypographyConfig) -> Self {
        Self { fonts, typo }
    }

    /// Root document container.
    pub fn document(&self) -> ResolvedStyle {
        let content_width = Pt::new(451.276); // A4 minus 72pt margins each side
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.width = Dimension::Length(content_width);
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.line_height = LineHeight::Number(self.typo.body_line_height);
        s.typography.color = Color::BLACK;
        s
    }

    /// Paragraph container.
    pub fn paragraph(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.line_height = LineHeight::Number(self.typo.body_line_height);
        s.typography.color = Color::BLACK;
        s
    }

    /// Heading container (h1–h6).
    pub fn heading(&self, level: u8) -> ResolvedStyle {
        let idx = (level.saturating_sub(1) as usize).min(5);
        let size = self.typo.heading_sizes[idx];

        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_top = Dimension::Length(self.typo.heading_spacing_above);
        s.layout.margin_bottom = Dimension::Length(self.typo.heading_spacing_below);
        s.typography.font_families = vec![self.fonts.heading.clone()];
        s.typography.font_size = size;
        s.typography.font_weight = 700;
        s.typography.line_height = LineHeight::Number(1.2);
        s.typography.color = Color::BLACK;
        s.fragmentation.keep_with_next = true;
        s
    }

    /// Inline text node from an `InlineStyle`.
    pub fn inline_text(&self, inherited: &InlineStyle) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Inline;
        s.typography.font_families = inherited.font_families.clone();
        s.typography.font_size = inherited.font_size;
        s.typography.font_weight = inherited.font_weight;
        s.typography.font_style = inherited.font_style;
        s.typography.text_decoration = inherited.text_decoration;
        s.typography.color = inherited.color;
        s.typography.line_height = LineHeight::Number(self.typo.body_line_height);
        if inherited.is_superscript {
            s.typography.vertical_align = VerticalAlign::Super;
        }
        s
    }

    /// Inline code (within a paragraph).
    pub fn inline_code(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Inline;
        s.typography.font_families = vec![self.fonts.mono.clone()];
        s.typography.font_size = Pt::new(self.typo.body_size.get() * 0.9);
        s.typography.color = Color::BLACK;
        s.visual.background_color = Some(Color::from_rgb8(246, 248, 250));
        s.layout.padding_left = LengthPercentage::Length(Pt::new(3.0));
        s.layout.padding_right = LengthPercentage::Length(Pt::new(3.0));
        s.layout.padding_top = LengthPercentage::Length(Pt::new(1.0));
        s.layout.padding_bottom = LengthPercentage::Length(Pt::new(1.0));
        s
    }

    /// Code block outer container.
    pub fn code_block(&self) -> ResolvedStyle {
        let pad = LengthPercentage::Length(self.typo.code_padding);
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.layout.padding_top = pad;
        s.layout.padding_bottom = pad;
        s.layout.padding_left = pad;
        s.layout.padding_right = pad;
        s.visual.background_color = Some(Color::from_rgb8(246, 248, 250));
        s.visual.border_top = border_thin();
        s.visual.border_bottom = border_thin();
        s.visual.border_left = border_thin();
        s.visual.border_right = border_thin();
        s.typography.font_families = vec![self.fonts.mono.clone()];
        s.typography.font_size = self.typo.code_size;
        s.typography.line_height = LineHeight::Number(self.typo.code_line_height);
        s.typography.white_space = WhiteSpace::Pre;
        s.typography.color = Color::BLACK;
        s.fragmentation.break_inside = oxipdf::ir::style::fragmentation::BreakInside::Avoid;
        s
    }

    /// Code block title bar (filename header above the code).
    pub fn code_title(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.layout.padding_left = LengthPercentage::Length(self.typo.code_padding);
        s.layout.padding_right = LengthPercentage::Length(self.typo.code_padding);
        s.layout.padding_top = LengthPercentage::Length(Pt::new(4.0));
        s.layout.padding_bottom = LengthPercentage::Length(Pt::new(4.0));
        s.visual.background_color = Some(Color::from_rgb8(230, 233, 237));
        s.visual.border_bottom = border_thin();
        s.typography.font_families = vec![self.fonts.mono.clone()];
        s.typography.font_size = Pt::new(self.typo.code_size.get() * 0.9);
        s.typography.font_weight = 600;
        s.typography.color = Color::from_rgb8(88, 96, 105);
        s
    }

    /// Blockquote container.
    pub fn blockquote(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_left = Dimension::Length(self.typo.blockquote_indent);
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.layout.padding_left = LengthPercentage::Length(Pt::new(12.0));
        s.visual.border_left = BorderSide {
            width: Pt::new(3.0),
            style: BorderStyle::Solid,
            color: Color::from_rgb8(208, 215, 222),
        };
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.font_style = FontStyle::Italic;
        s.typography.color = Color::from_rgb8(88, 96, 105);
        s
    }

    /// List container (ul / ol).
    pub fn list(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s
    }

    /// List item container — no typography set (avoids taffy inflating container height
    /// from font metrics). Typography comes from the inline text children.
    pub fn list_item(&self, ordered: bool, index: u32) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_left = Dimension::Length(self.typo.list_indent);
        s.layout.margin_bottom = Dimension::Length(Pt::ZERO);
        // Don't set font_size/line_height on the container — let children define height
        s.typography.font_size = Pt::ZERO;
        s.typography.line_height = LineHeight::Length(Pt::ZERO);
        s.list.marker = Some(if ordered {
            ListMarker::Decimal(index)
        } else {
            ListMarker::Disc
        });
        s.list.position = ListMarkerPosition::Outside;
        s
    }

    /// Thematic break (horizontal rule).
    pub fn thematic_break(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.height = Dimension::Length(Pt::new(1.0));
        s.layout.margin_top = Dimension::Length(Pt::new(16.0));
        s.layout.margin_bottom = Dimension::Length(Pt::new(16.0));
        s.visual.border_top = BorderSide {
            width: Pt::new(0.5),
            style: BorderStyle::Solid,
            color: Color::from_rgb8(208, 215, 222),
        };
        s
    }

    /// Table cell style.
    pub fn table_cell(&self, is_header: bool) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.padding_top = LengthPercentage::Length(Pt::new(4.0));
        s.layout.padding_bottom = LengthPercentage::Length(Pt::new(4.0));
        s.layout.padding_left = LengthPercentage::Length(Pt::new(8.0));
        s.layout.padding_right = LengthPercentage::Length(Pt::new(8.0));
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.line_height = LineHeight::Number(self.typo.body_line_height);
        s.typography.color = Color::BLACK;
        if is_header {
            s.typography.font_weight = 700;
        }
        s
    }

    /// Definition term (bold).
    pub fn definition_term(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_top = Dimension::Length(Pt::new(8.0));
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.font_weight = 700;
        s.typography.color = Color::BLACK;
        s.fragmentation.keep_with_next = true;
        s
    }

    /// Definition description (indented).
    pub fn definition_description(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_left = Dimension::Length(Pt::new(24.0));
        s.layout.margin_bottom = Dimension::Length(Pt::new(4.0));
        s.typography.font_families = vec![self.fonts.body.clone()];
        s.typography.font_size = self.typo.body_size;
        s.typography.line_height = LineHeight::Number(self.typo.body_line_height);
        s.typography.color = Color::BLACK;
        s
    }

    /// Image wrapper (centered, with max width).
    pub fn image(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.typography.text_align = TextAlign::Center;
        s
    }

    /// Link style (blue, underlined).
    pub fn link(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Inline;
        s.typography.color = Color::from_rgb8(3, 102, 214);
        s.typography.text_decoration = TextDecoration::Underline;
        s
    }

    /// Error block (red text).
    pub fn error(&self) -> ResolvedStyle {
        let mut s = ResolvedStyle::default();
        s.layout.display = Display::Block;
        s.layout.margin_bottom = Dimension::Length(self.typo.paragraph_spacing);
        s.layout.padding_left = LengthPercentage::Length(Pt::new(8.0));
        s.layout.padding_right = LengthPercentage::Length(Pt::new(8.0));
        s.layout.padding_top = LengthPercentage::Length(Pt::new(4.0));
        s.layout.padding_bottom = LengthPercentage::Length(Pt::new(4.0));
        s.visual.background_color = Some(Color::from_rgb8(255, 240, 240));
        s.typography.font_families = vec![self.fonts.mono.clone()];
        s.typography.font_size = self.typo.code_size;
        s.typography.color = Color::from_rgb8(203, 36, 49);
        s
    }
}

/// Syntax highlighting token colors for code blocks.
pub fn token_color(kind: oxidoc_highlight::token::TokenKind) -> Color {
    use oxidoc_highlight::token::TokenKind;
    match kind {
        TokenKind::Keyword => Color::from_rgb8(215, 58, 73), // red
        TokenKind::String => Color::from_rgb8(3, 47, 98),    // dark blue
        TokenKind::Comment => Color::from_rgb8(106, 115, 125), // gray
        TokenKind::Number => Color::from_rgb8(0, 92, 197),   // blue
        TokenKind::Function => Color::from_rgb8(111, 66, 193), // purple
        TokenKind::Type => Color::from_rgb8(227, 98, 9),     // orange
        TokenKind::Operator => Color::from_rgb8(215, 58, 73), // red
        TokenKind::Punctuation => Color::BLACK,
        TokenKind::Property => Color::from_rgb8(0, 92, 197), // blue
        TokenKind::Builtin => Color::from_rgb8(0, 92, 197),  // blue
        TokenKind::Attr => Color::from_rgb8(227, 98, 9),     // orange
        TokenKind::Variable => Color::from_rgb8(36, 41, 46), // dark
        TokenKind::Plain => Color::from_rgb8(36, 41, 46),    // dark
    }
}

fn border_thin() -> BorderSide {
    BorderSide {
        width: Pt::new(0.5),
        style: BorderStyle::Solid,
        color: Color::from_rgb8(225, 228, 232),
    }
}
