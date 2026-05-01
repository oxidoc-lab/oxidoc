pub mod button;
pub mod config;
pub mod emit;
pub mod frontmatter;
pub mod resolve;

pub use button::render_copy_markdown_button;
pub use config::{LlmConfig, LlmPathOverride, ResolvedLlm};
pub use emit::{PageLlmInput, generate_llm_outputs, top_level_segment};
pub use frontmatter::{PageLlmFrontmatter, extract_page_llm};
pub use resolve::resolve_llm_for_page;
