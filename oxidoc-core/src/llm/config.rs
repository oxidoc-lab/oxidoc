use serde::Deserialize;

/// Site-wide LLM export configuration (`[llm]` in oxidoc.toml).
#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub copy_button: bool,
    /// When true, emit `/<section>/llms.txt` + `/<section>/llms-full.txt`
    /// for each top-level URL section.
    #[serde(default = "default_true")]
    pub section_files: bool,
    /// Path-prefix overrides. Longest matching prefix wins.
    #[serde(default)]
    pub paths: Vec<LlmPathOverride>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            copy_button: true,
            section_files: true,
            paths: Vec::new(),
        }
    }
}

/// Per-path override. Any field left unset inherits from the level above.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmPathOverride {
    /// URL path prefix (e.g. `"docs"`, `"docs/internal"`). No leading slash.
    pub path: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub copy_button: Option<bool>,
}

/// Resolved per-page LLM settings after cascade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedLlm {
    pub enabled: bool,
    pub copy_button: bool,
}

fn default_true() -> bool {
    true
}
