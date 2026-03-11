//! SemanticSearch island component for Wasm hydration.

use oxidoc_island::{IslandError, OxidocIsland};

pub struct SemanticSearch;

impl OxidocIsland for SemanticSearch {
    fn island_type() -> &'static str {
        "semantic-search"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        let _props: serde_json::Value = serde_json::from_str(props_json)?;

        let html = r#"<div class="oxidoc-search-container" style="padding: 16px; border: 1px solid #e0e0e0; border-radius: 8px; background: #f9f9f9;">
            <input type="text" placeholder="Search..." class="oxidoc-search-input" style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; margin-bottom: 12px;">
            <div class="oxidoc-search-results" style="max-height: 400px; overflow-y: auto;"></div>
            <div class="oxidoc-search-placeholder" style="color: #999; text-align: center; padding: 20px;">Semantic search engine loaded. Type to search.</div>
        </div>"#;

        target.set_inner_html(html);
        Ok(())
    }
}
