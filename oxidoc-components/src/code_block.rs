use oxidoc_island::OxidocIsland;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CodeBlockProps {
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub code: String,
}

pub struct CodeBlock;

impl OxidocIsland for CodeBlock {
    fn island_type() -> &'static str {
        "codeblock"
    }

    fn mount(target: web_sys::Element, props_json: &str) {
        let _props: CodeBlockProps = serde_json::from_str(props_json).unwrap_or_default();
        let _ = target;
    }
}
