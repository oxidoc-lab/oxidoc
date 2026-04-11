use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrintError {
    #[error("Failed to read file: {path}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Image not found: {path}")]
    ImageNotFound { path: String },

    #[error("Unsupported image format: {path}")]
    UnsupportedImageFormat { path: String },

    #[error("IR tree construction failed: {0}")]
    TreeBuild(#[from] oxipdf::ir::InputValidationError),

    #[error("PDF rendering failed: {0}")]
    Render(#[from] oxipdf::RenderError),

    #[error("No fonts available — at least one font family must be resolvable")]
    NoFonts,
}

pub type Result<T> = std::result::Result<T, PrintError>;
