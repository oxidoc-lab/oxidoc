// False positive from thiserror's derive macro on struct variant fields.
#![allow(unused_assignments)]

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum OxidocError {
    #[error("Failed to read config file: {path}")]
    #[diagnostic(
        code(oxidoc::config::read),
        help("Ensure oxidoc.toml exists in your project root")
    )]
    ConfigRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid config: {message}")]
    #[diagnostic(code(oxidoc::config::parse), help("Check your oxidoc.toml syntax"))]
    ConfigParse {
        message: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("Missing required field `project.name` in oxidoc.toml")]
    #[diagnostic(
        code(oxidoc::config::missing_name),
        help("Add [project] section with a `name` field to oxidoc.toml")
    )]
    ConfigMissingName,

    #[error("Docs directory not found: {path}")]
    #[diagnostic(
        code(oxidoc::crawler::no_docs),
        help("Create a docs/ directory with .rdx files, or set routing.navigation in oxidoc.toml")
    )]
    DocsNotFound { path: String },

    #[error("Page not found: {slug}")]
    #[diagnostic(
        code(oxidoc::crawler::page_not_found),
        help("Check that the file exists in your docs/ directory")
    )]
    PageNotFound { slug: String },

    #[error("Failed to read file: {path}")]
    #[diagnostic(code(oxidoc::io::read))]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    #[diagnostic(code(oxidoc::io::write))]
    FileWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("RDX parse error in {path}: {message}")]
    #[diagnostic(code(oxidoc::rdx::parse))]
    RdxParse { path: String, message: String },

    #[error("Internal error: path {path} is not under root {root}")]
    #[diagnostic(code(oxidoc::internal::path_prefix))]
    PathNotUnderRoot { path: String, root: String },

    #[error("Failed to create directory: {path}")]
    #[diagnostic(code(oxidoc::io::mkdir))]
    DirCreate {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Search index error: {message}")]
    #[diagnostic(code(oxidoc::search::index))]
    SearchIndex { message: String },

    #[error("Translation file not found: {path}")]
    #[diagnostic(code(oxidoc::i18n::not_found))]
    TranslationNotFound { path: String },

    #[error("Translation parse error in {path}: {message}")]
    #[diagnostic(code(oxidoc::i18n::parse))]
    TranslationParse { path: String, message: String },

    #[error("Search provider '{provider}' requires field '{field}'")]
    #[diagnostic(
        code(oxidoc::search::config),
        help("Add {field} to your [search] config in oxidoc.toml")
    )]
    SearchProviderConfig { provider: String, field: String },

    #[error("Failed to read theme file: {path}")]
    #[diagnostic(code(oxidoc::theme::read))]
    ThemeFileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid theme file {path}: {message}")]
    #[diagnostic(code(oxidoc::theme::parse), help("Check your theme .toml file syntax"))]
    ThemeParse { path: String, message: String },
}

impl OxidocError {
    /// Whether this is a configuration-related error (for exit code classification).
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            OxidocError::ConfigRead { .. }
                | OxidocError::ConfigParse { .. }
                | OxidocError::ConfigMissingName
        )
    }
}

pub type Result<T> = std::result::Result<T, OxidocError>;
