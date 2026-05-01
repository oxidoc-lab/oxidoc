use serde_json::Value;

/// Per-page LLM overrides extracted from frontmatter (`llm:` key).
///
/// Accepts either a bool shorthand (`llm: false`) or a table
/// (`llm: { enabled: false, copy_button: true }`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageLlmFrontmatter {
    pub enabled: Option<bool>,
    pub copy_button: Option<bool>,
}

impl PageLlmFrontmatter {
    /// Returns true if no fields were set (page inherits everything).
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.copy_button.is_none()
    }
}

/// Extract the `llm` key from RDX frontmatter.
///
/// Returns `None` if the key is absent. Returns `Some(empty)` if the value is
/// neither a bool nor a table (which we silently ignore — invalid frontmatter
/// shouldn't break the build).
pub fn extract_page_llm(root: &rdx_ast::Root) -> Option<PageLlmFrontmatter> {
    let value = root.frontmatter.as_ref()?.get("llm")?;
    Some(parse_value(value))
}

fn parse_value(value: &Value) -> PageLlmFrontmatter {
    match value {
        Value::Bool(b) => PageLlmFrontmatter {
            enabled: Some(*b),
            ..Default::default()
        },
        Value::Object(map) => PageLlmFrontmatter {
            enabled: map.get("enabled").and_then(Value::as_bool),
            copy_button: map.get("copy_button").and_then(Value::as_bool),
        },
        _ => PageLlmFrontmatter::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_root(fm: Value) -> rdx_ast::Root {
        let mut root = rdx_parser::parse("");
        root.frontmatter = Some(fm);
        root
    }

    fn make_root_no_fm() -> rdx_ast::Root {
        let mut root = rdx_parser::parse("");
        root.frontmatter = None;
        root
    }

    #[test]
    fn missing_llm_key_returns_none() {
        let root = make_root(json!({"title": "T"}));
        assert!(extract_page_llm(&root).is_none());
    }

    #[test]
    fn no_frontmatter_returns_none() {
        let root = make_root_no_fm();
        assert!(extract_page_llm(&root).is_none());
    }

    #[test]
    fn bool_shorthand_false() {
        let root = make_root(json!({"llm": false}));
        let fm = extract_page_llm(&root).unwrap();
        assert_eq!(fm.enabled, Some(false));
        assert_eq!(fm.copy_button, None);
    }

    #[test]
    fn table_form_partial() {
        let root = make_root(json!({"llm": {"copy_button": false}}));
        let fm = extract_page_llm(&root).unwrap();
        assert_eq!(fm.enabled, None);
        assert_eq!(fm.copy_button, Some(false));
    }

    #[test]
    fn invalid_value_yields_empty() {
        let root = make_root(json!({"llm": "yes"}));
        let fm = extract_page_llm(&root).unwrap();
        assert!(fm.is_empty());
    }
}
