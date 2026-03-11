use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccordionProps {
    #[serde(default)]
    pub items: Vec<AccordionItem>,
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccordionItem {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub open: bool,
}

pub struct Accordion;

impl OxidocIsland for Accordion {
    fn island_type() -> &'static str {
        "accordion"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, accordion_view)
    }
}

fn accordion_view(props: AccordionProps) -> impl IntoView {
    let multiple = props.multiple;
    let open_states: Vec<RwSignal<bool>> = props
        .items
        .iter()
        .map(|item| RwSignal::new(item.open))
        .collect();

    let item_views: Vec<_> = props
        .items
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let is_open = open_states[i];
            let states = open_states.clone();
            let states2 = open_states.clone();
            view! {
                <div class="oxidoc-accordion-item">
                    <button
                        class="oxidoc-accordion-trigger"
                        role="button"
                        aria-expanded=move || is_open.get().to_string()
                        on:click=move |_| {
                            if !multiple {
                                for (j, s) in states.iter().enumerate() {
                                    if j != i { s.set(false); }
                                }
                            }
                            is_open.update(|v| *v = !*v);
                        }
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" || ev.key() == " " {
                                ev.prevent_default();
                                if !multiple {
                                    for (j, s) in states2.iter().enumerate() {
                                        if j != i { s.set(false); }
                                    }
                                }
                                is_open.update(|v| *v = !*v);
                            }
                        }
                    >
                        <span class="oxidoc-accordion-title">{item.title.clone()}</span>
                        <span
                            class="oxidoc-accordion-chevron"
                            aria-hidden="true"
                            style=move || if is_open.get() { "transform:rotate(90deg)" } else { "" }
                        >
                            "\u{25b6}"
                        </span>
                    </button>
                    <div
                        class="oxidoc-accordion-content"
                        role="region"
                        style=move || if is_open.get() { "" } else { "display:none" }
                    >
                        // SAFETY: content is pre-sanitized by oxidoc-core during build.
                        <div inner_html=item.content.clone()></div>
                    </div>
                </div>
            }
        })
        .collect();

    view! {
        <div class="oxidoc-accordion" role="presentation">
            {item_views}
        </div>
    }
}
