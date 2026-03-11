use oxidoc_island::OxidocIsland;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AccordionProps {
    #[serde(default)]
    pub title: String,
}

pub struct Accordion;

impl OxidocIsland for Accordion {
    fn island_type() -> &'static str {
        "accordion"
    }

    fn mount(target: web_sys::Element, props_json: &str) {
        let _props: AccordionProps = serde_json::from_str(props_json).unwrap_or_default();
        let _ = target;
    }
}
