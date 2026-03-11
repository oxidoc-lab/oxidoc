use oxidoc_island::OxidocIsland;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TabsProps {
    #[serde(default)]
    pub labels: Vec<String>,
}

pub struct Tabs;

impl OxidocIsland for Tabs {
    fn island_type() -> &'static str {
        "tabs"
    }

    fn mount(target: web_sys::Element, props_json: &str) {
        let _props: TabsProps = serde_json::from_str(props_json).unwrap_or_default();
        let _ = target;
    }
}
