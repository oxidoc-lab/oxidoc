use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabsProps {
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub contents: Vec<String>,
    #[serde(default)]
    pub storage_key: Option<String>,
}

pub struct Tabs;

impl OxidocIsland for Tabs {
    fn island_type() -> &'static str {
        "tabs"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, tabs_view)
    }
}

fn tabs_view(props: TabsProps) -> impl IntoView {
    let initial = props
        .storage_key
        .as_ref()
        .and_then(|key| {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(key).ok().flatten())
                .and_then(|v| v.parse::<usize>().ok())
        })
        .unwrap_or(0);

    let active = RwSignal::new(initial);
    let storage_key = props.storage_key.clone();
    let tab_count = props.labels.len();

    let tab_buttons: Vec<_> = props
        .labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let label = label.clone();
            let sk = storage_key.clone();
            let btn = view! {
                <button
                    class=move || if active.get() == i { "oxidoc-tab active" } else { "oxidoc-tab" }
                    role="tab"
                    aria-selected=move || (active.get() == i).to_string()
                    tabindex=move || if active.get() == i { "0" } else { "-1" }
                    on:click=move |_| {
                        active.set(i);
                        if let Some(ref key) = sk
                            && let Some(storage) = web_sys::window()
                                .and_then(|w| w.local_storage().ok().flatten())
                        {
                            let _ = storage.set_item(key, &i.to_string());
                        }
                    }
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        match ev.key().as_str() {
                            "ArrowRight" => { ev.prevent_default(); active.set((active.get() + 1) % tab_count); },
                            "ArrowLeft" => { ev.prevent_default(); active.set(active.get().checked_sub(1).unwrap_or(tab_count - 1)); },
                            "Home" => { ev.prevent_default(); active.set(0); },
                            "End" => { ev.prevent_default(); active.set(tab_count.saturating_sub(1)); },
                            _ => {}
                        }
                    }
                >
                    {label}
                </button>
            };
            btn
        })
        .collect();

    let tab_panels: Vec<_> = props
        .contents
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let content = content.clone();
            view! {
                <div
                    class="oxidoc-tab-panel"
                    role="tabpanel"
                    style=move || if active.get() == i { "" } else { "display:none" }
                >
                    // SAFETY: content is pre-sanitized by oxidoc-core during build.
                    <div inner_html=content.clone()></div>
                </div>
            }
        })
        .collect();

    view! {
        <div class="oxidoc-tabs" role="tablist">
            <div class="oxidoc-tabs-list">
                {tab_buttons}
            </div>
            <div class="oxidoc-tabs-panels">
                {tab_panels}
            </div>
        </div>
    }
}
