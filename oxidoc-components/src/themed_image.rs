use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemedImageProps {
    #[serde(default)]
    pub light: String,
    #[serde(default)]
    pub dark: String,
    #[serde(default)]
    pub alt: String,
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub height: Option<String>,
}

pub struct ThemedImage;

impl OxidocIsland for ThemedImage {
    fn island_type() -> &'static str {
        "themedimage"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, themed_image_view)
    }
}

fn themed_image_view(props: ThemedImageProps) -> impl IntoView {
    view! {
        <picture class="oxidoc-themed-image">
            <source media="(prefers-color-scheme: dark)" srcset=props.dark.clone() />
            <img
                src=props.light
                alt=props.alt
                width=props.width.unwrap_or_default()
                height=props.height.unwrap_or_default()
                loading="lazy"
            />
        </picture>
    }
}
