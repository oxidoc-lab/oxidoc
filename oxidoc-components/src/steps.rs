use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepsProps {
    #[serde(default)]
    pub steps: Vec<StepProps>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
}

pub struct Steps;

impl OxidocIsland for Steps {
    fn island_type() -> &'static str {
        "steps"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, steps_view)
    }
}

fn steps_view(props: StepsProps) -> impl IntoView {
    let step_views: Vec<_> = props
        .steps
        .into_iter()
        .enumerate()
        .map(|(i, step)| {
            view! {
                <div class="oxidoc-step">
                    <div class="oxidoc-step-indicator">
                        <span class="oxidoc-step-number">{i + 1}</span>
                    </div>
                    <div class="oxidoc-step-content">
                        <h3 class="oxidoc-step-title">{step.title}</h3>
                        <div class="oxidoc-step-body" inner_html=step.content></div>
                    </div>
                </div>
            }
        })
        .collect();

    view! {
        <div class="oxidoc-steps">
            {step_views}
        </div>
    }
}
