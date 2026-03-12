use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeBlockProps {
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub code: String,
    /// Pre-highlighted HTML from build time
    #[serde(default)]
    pub code_html: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub line_numbers: bool,
    #[serde(default)]
    pub highlight_lines: Vec<usize>,
}

pub struct CodeBlock;

impl OxidocIsland for CodeBlock {
    fn island_type() -> &'static str {
        "codeblock"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        crate::mount_component(target, props_json, code_block_view)
    }
}

fn code_block_view(props: CodeBlockProps) -> impl IntoView {
    let copied = RwSignal::new(false);
    let code_for_copy = props.code.clone();

    let has_filename = !props.filename.is_empty();
    let has_language = !props.language.is_empty();
    let filename = props.filename.clone();
    let language_label = props.language.clone();
    let language_class = format!("language-{}", props.language);

    // Use pre-highlighted HTML from build time
    let code_html = props.code_html.clone();

    view! {
        <div class="oxidoc-codeblock">
            {has_filename.then(|| view! {
                <div class="oxidoc-codeblock-header">
                    <span>{filename.clone()}</span>
                    {has_language.then(|| view! {
                        <span>{language_label.clone()}</span>
                    })}
                </div>
            })}
            <div class="oxidoc-codeblock-body">
                <pre>
                    <code class=language_class inner_html=code_html></code>
                    <button
                        class=move || if copied.get() { "oxidoc-copy-code copied" } else { "oxidoc-copy-code" }
                        on:click={
                            let code = code_for_copy.clone();
                            move |_| {
                                if let Some(window) = web_sys::window() {
                                    let clipboard = window.navigator().clipboard();
                                    let _ = clipboard.write_text(&code);
                                    copied.set(true);
                                    leptos::prelude::set_timeout(
                                        move || copied.set(false),
                                        std::time::Duration::from_secs(2),
                                    );
                                }
                            }
                        }
                    >
                        {move || if copied.get() { "\u{2713} Copied" } else { "Copy" }}
                    </button>
                </pre>
            </div>
        </div>
    }
}
