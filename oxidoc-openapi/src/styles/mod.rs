//! Scoped CSS styles for the API Playground component with Shadow DOM isolation.

mod controls;
mod layout;

/// Returns the complete CSS string for the API Playground Shadow DOM.
pub fn get_styles() -> String {
    format!("{}\n{}", layout::CSS, controls::CSS)
}
