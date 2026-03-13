use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BadgeProps {
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub outline: bool,
}

fn default_kind() -> String {
    "info".to_string()
}

pub struct Badge;

impl OxidocIsland for Badge {
    fn island_type() -> &'static str {
        "badge"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, badge_view)
    }
}

fn badge_view(props: BadgeProps) -> impl IntoView {
    let class = format!(
        "oxidoc-badge oxidoc-badge-{}{}",
        props.kind,
        if props.outline {
            " oxidoc-badge-outline"
        } else {
            ""
        }
    );
    view! {
        <span class=class>{props.text}</span>
    }
}
