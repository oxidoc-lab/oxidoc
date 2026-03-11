use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardGridProps {
    #[serde(default = "default_columns")]
    pub columns: u8,
    #[serde(default)]
    pub cards: Vec<CardProps>,
}

fn default_columns() -> u8 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

pub struct CardGrid;

impl OxidocIsland for CardGrid {
    fn island_type() -> &'static str {
        "cardgrid"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, card_grid_view)
    }
}

fn card_grid_view(props: CardGridProps) -> impl IntoView {
    let cols = props.columns.clamp(1, 6);
    let style = format!("display:grid;grid-template-columns:repeat({cols},1fr);gap:1rem");

    let card_views: Vec<_> = props
        .cards
        .into_iter()
        .map(|card| {
            let has_icon = card.icon.is_some();
            let icon = card.icon.clone().unwrap_or_default();
            let has_desc = !card.description.is_empty();
            let title = card.title.clone();
            let description = card.description.clone();

            let inner = view! {
                <div class="oxidoc-card-inner">
                    {has_icon.then(|| view! {
                        <span class="oxidoc-card-icon" aria-hidden="true">{icon.clone()}</span>
                    })}
                    <h3 class="oxidoc-card-title">{title}</h3>
                    {has_desc.then(|| view! {
                        <p class="oxidoc-card-desc">{description}</p>
                    })}
                </div>
            };

            if let Some(href) = &card.href {
                view! {
                    <a class="oxidoc-card" href=href.clone()>
                        {inner}
                    </a>
                }
                .into_any()
            } else {
                view! {
                    <div class="oxidoc-card">
                        {inner}
                    </div>
                }
                .into_any()
            }
        })
        .collect();

    view! {
        <div class="oxidoc-cardgrid" style=style>
            {card_views}
        </div>
    }
}
