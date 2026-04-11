use oxipdf::ir::units::Pt;
use std::path::PathBuf;

/// Top-level configuration for the print pipeline.
pub struct PrintConfig {
    pub page: PageConfig,
    pub fonts: FontConfig,
    pub typography: TypographyConfig,
    /// Root directory for resolving relative asset paths (images, bib files).
    pub project_root: PathBuf,
    /// Debug mode: draw colored borders on every element showing its computed box.
    pub debug_boxes: bool,
}

impl PrintConfig {
    /// Sensible defaults for A4 academic documents.
    pub fn default_with_root(project_root: PathBuf) -> Self {
        Self {
            page: PageConfig::default(),
            fonts: FontConfig::default(),
            typography: TypographyConfig::default(),
            project_root,
            debug_boxes: false,
        }
    }
}

/// Page dimensions and margins.
pub struct PageConfig {
    pub size: PageSize,
    /// Margins in points. For recto/verso, `left` is inner (binding) and `right` is outer.
    pub margin_top: Pt,
    pub margin_bottom: Pt,
    pub margin_inner: Pt,
    pub margin_outer: Pt,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            size: PageSize::A4,
            margin_top: Pt::new(72.0), // 1 inch
            margin_bottom: Pt::new(72.0),
            margin_inner: Pt::new(72.0),
            margin_outer: Pt::new(72.0),
        }
    }
}

/// Standard page sizes.
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    A4,
    Letter,
    /// Custom size in points.
    Custom {
        width: Pt,
        height: Pt,
    },
}

impl PageSize {
    pub fn width(self) -> Pt {
        match self {
            Self::A4 => Pt::new(595.276),
            Self::Letter => Pt::new(612.0),
            Self::Custom { width, .. } => width,
        }
    }

    pub fn height(self) -> Pt {
        match self {
            Self::A4 => Pt::new(841.890),
            Self::Letter => Pt::new(792.0),
            Self::Custom { height, .. } => height,
        }
    }
}

/// Font family configuration.
pub struct FontConfig {
    pub body: String,
    pub heading: String,
    pub mono: String,
    /// Additional directories to search for font files.
    pub font_dirs: Vec<PathBuf>,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            body: "Noto Serif".into(),
            heading: "Noto Sans".into(),
            mono: "Noto Sans Mono".into(),
            font_dirs: Vec::new(),
        }
    }
}

/// Typography settings — font sizes, spacing, colors for each element type.
pub struct TypographyConfig {
    pub body_size: Pt,
    pub body_line_height: f64,
    pub heading_sizes: [Pt; 6],
    pub code_size: Pt,
    pub code_line_height: f64,
    /// Paragraph spacing (margin-bottom).
    pub paragraph_spacing: Pt,
    /// Heading spacing above.
    pub heading_spacing_above: Pt,
    /// Heading spacing below.
    pub heading_spacing_below: Pt,
    /// List item indent per nesting level.
    pub list_indent: Pt,
    /// Blockquote left margin.
    pub blockquote_indent: Pt,
    /// Code block padding.
    pub code_padding: Pt,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            body_size: Pt::new(11.0),
            body_line_height: 1.4,
            heading_sizes: [
                Pt::new(26.0), // h1
                Pt::new(20.0), // h2
                Pt::new(16.0), // h3
                Pt::new(13.0), // h4
                Pt::new(11.0), // h5
                Pt::new(11.0), // h6
            ],
            code_size: Pt::new(9.0),
            code_line_height: 1.4,
            paragraph_spacing: Pt::new(6.0),
            heading_spacing_above: Pt::new(12.0),
            heading_spacing_below: Pt::new(4.0),
            list_indent: Pt::new(24.0),
            blockquote_indent: Pt::new(24.0),
            code_padding: Pt::new(10.0),
        }
    }
}
