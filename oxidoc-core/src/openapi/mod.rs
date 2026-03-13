pub mod html;
pub mod nav;
pub mod parser;
pub mod types;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use html::render_endpoint_html;
pub use nav::{
    ApiBuildContext, build_api_pages, endpoint_slug, generate_api_nav_groups,
    group_endpoints_by_tag,
};
pub use parser::{extract_endpoints, load_openapi_spec};
pub use types::{ApiEndpoint, ApiParameter, ApiRequestBody, ApiResponse, SchemaField};
