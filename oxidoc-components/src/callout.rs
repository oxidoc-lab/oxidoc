use oxidoc_island::OxidocIsland;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CalloutProps {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub content: String,
}

pub struct Callout;

impl OxidocIsland for Callout {
    fn island_type() -> &'static str {
        "callout"
    }

    fn mount(target: web_sys::Element, props_json: &str) {
        let _props: CalloutProps = serde_json::from_str(props_json).unwrap_or_default();
        // TODO: mount Leptos component into target
        let _ = target;
    }
}
