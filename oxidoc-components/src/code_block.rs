use leptos::prelude::*;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeBlockProps {
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub code: String,
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

    let lines: Vec<(usize, String)> = props
        .code
        .lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line.to_string()))
        .collect();

    let highlight_lines = props.highlight_lines.clone();
    let show_line_numbers = props.line_numbers;

    let has_filename = !props.filename.is_empty();
    let has_language = !props.language.is_empty();
    let filename = props.filename.clone();
    let language_label = props.language.clone();
    let language_class = format!("language-{}", props.language);

    let line_views: Vec<_> = lines
        .into_iter()
        .map(|(num, line)| {
            let highlighted = highlight_lines.contains(&num);
            let class = if highlighted {
                "oxidoc-line highlighted"
            } else {
                "oxidoc-line"
            };
            view! {
                <span class=class>
                    {show_line_numbers.then(|| view! {
                        <span class="oxidoc-line-number">{num}</span>
                    })}
                    <span class="oxidoc-line-content">{line}{"\n"}</span>
                </span>
            }
        })
        .collect();

    view! {
        <div class="oxidoc-codeblock">
            {has_filename.then(|| view! {
                <div class="oxidoc-codeblock-header">
                    <span class="oxidoc-codeblock-filename">{filename.clone()}</span>
                    {has_language.then(|| view! {
                        <span class="oxidoc-codeblock-lang">{language_label.clone()}</span>
                    })}
                </div>
            })}
            <div class="oxidoc-codeblock-body">
                <button
                    class=move || if copied.get() { "oxidoc-copy-btn copied" } else { "oxidoc-copy-btn" }
                    aria-label="Copy code"
                    title="Copy to clipboard"
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
                <pre><code class=language_class>
                    {line_views}
                </code></pre>
            </div>
        </div>
    }
}
