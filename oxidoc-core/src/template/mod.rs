mod meta;
mod nav;
mod page;
mod strings;

pub(crate) use meta::remove_overridden_meta_tags;
pub(crate) use nav::{
    build_header_actions, build_header_nav, build_menu_toggle, build_mobile_nav_links,
    render_logo_html,
};
pub use page::render_page;
pub(crate) use strings::{
    API_TABS_JS, BACK_TO_TOP_JS, COPY_MARKDOWN_JS, ERROR_404_HTML, HEADER_ACTIONS_HTML,
    HEADER_SCROLL_JS, MOBILE_MENU_JS, SCROLLSPY_JS, SEARCH_DIALOG_HTML, SEARCH_DIALOG_JS,
    THEME_TOGGLE_JS,
};
