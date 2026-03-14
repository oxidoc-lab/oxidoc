use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalloutProps {
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub collapsible: bool,
}

fn default_kind() -> String {
    "info".into()
}

pub struct Callout;

impl OxidocIsland for Callout {
    fn island_type() -> &'static str {
        "callout"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, callout_view)
    }
}

fn callout_view(props: CalloutProps) -> impl IntoView {
    let (icon, aria_label) = match props.kind.as_str() {
        "warning" => ("\u{26a0}\u{fe0f}", "Warning"),
        "error" | "danger" => ("\u{274c}", "Error"),
        "tip" | "success" => ("\u{2705}", "Tip"),
        _ => ("\u{2139}\u{fe0f}", "Information"),
    };

    let title = if props.title.is_empty() {
        aria_label.to_string()
    } else {
        props.title.clone()
    };

    let collapsed = RwSignal::new(props.collapsible);
    let collapsible = props.collapsible;

    let class = format!("oxidoc-callout oxidoc-callout-{}", props.kind);

    view! {
        <div class=class role="note" aria-label=aria_label>
            <div
                class="oxidoc-callout-header"
                on:click=move |_| {
                    if collapsible {
                        collapsed.update(|v| *v = !*v);
                    }
                }
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if collapsible && (ev.key() == "Enter" || ev.key() == " ") {
                        ev.prevent_default();
                        collapsed.update(|v| *v = !*v);
                    }
                }
            >
                <span class="oxidoc-callout-icon" aria-hidden="true">{icon}</span>
                <span class="oxidoc-callout-title">{title}</span>
            </div>
            <div
                class=move || if collapsed.get() { "oxidoc-callout-body oxidoc-collapsed" } else { "oxidoc-callout-body" }
            >
                // SAFETY: content is pre-sanitized by oxidoc-core during build.
                <div inner_html=props.content.clone()></div>
            </div>
        </div>
    }
}
