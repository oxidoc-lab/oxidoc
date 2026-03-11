use oxidoc_island::OxidocIsland;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CardGridProps {
    #[serde(default)]
    pub columns: Option<u8>,
}

pub struct CardGrid;

impl OxidocIsland for CardGrid {
    fn island_type() -> &'static str {
        "cardgrid"
    }

    fn mount(target: web_sys::Element, props_json: &str) {
        let _props: CardGridProps = serde_json::from_str(props_json).unwrap_or_default();
        let _ = target;
    }
}
