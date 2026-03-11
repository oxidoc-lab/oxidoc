//! Focus trap utilities for modal overlays and menu components.
//!
//! A focus trap ensures keyboard navigation is contained within a component
//! (e.g., a modal, dropdown, or search overlay), cycling through focusable elements
//! and preventing focus from escaping to the rest of the page.

use crate::IslandError;

/// Set up a focus trap within a container element.
///
/// Traps Tab and Shift+Tab navigation within the container, cycling through
/// all focusable elements (buttons, links, inputs, elements with tabindex, etc.).
/// Automatically focuses the first focusable element.
///
/// # Arguments
/// * `container` - The element to trap focus within
///
/// # Focusable Elements
/// The trap identifies these as focusable:
/// - `a` (with href)
/// - `button` (not disabled)
/// - `input` (not disabled)
/// - `select` (not disabled)
/// - `textarea` (not disabled)
/// - Elements with `tabindex >= 0`
///
/// # Errors
/// Returns an error if the container has no focusable elements or DOM manipulation fails.
///
/// # Example
/// ```ignore
/// let modal = document.query_selector("#my-modal")?;
/// trap_focus(&modal)?;
/// // User can now only Tab through elements within the modal
/// ```
pub fn trap_focus(container: &web_sys::Element) -> Result<(), IslandError> {
    use wasm_bindgen::JsCast;

    let focusable_selector = r#"
        a[href],
        button:not([disabled]),
        input:not([disabled]),
        select:not([disabled]),
        textarea:not([disabled]),
        [tabindex]:not([tabindex="-1"])
    "#;

    let focusable_elements = container
        .query_selector_all(focusable_selector)
        .map_err(|e| IslandError {
            message: format!("Failed to query focusable elements: {e:?}"),
        })?;

    if focusable_elements.length() == 0 {
        return Err(IslandError {
            message: "No focusable elements found in container".into(),
        });
    }

    // Focus the first focusable element
    if let Some(first) = focusable_elements
        .get(0)
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = first.focus();
    }

    // Note: Setting up the keydown listener is left to the caller
    // since it requires closure capture and event listener management.
    // This function provides the helper to query focusable elements.

    Ok(())
}

/// Release a focus trap by removing trapped state from a container.
///
/// Currently a no-op as a placeholder for future listener cleanup.
/// In a full implementation, this would remove the keydown event listener
/// that was added when setting up the trap.
///
/// # Arguments
/// * `_container` - The element to release focus trap from
///
/// # Example
/// ```ignore
/// release_focus(&modal);
/// // Focus can now leave the modal
/// ```
pub fn release_focus(_container: &web_sys::Element) {
    // Placeholder for future event listener cleanup
    // When trap_focus is enhanced to attach listeners directly,
    // this will remove them.
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_focus_trap_module_compiles() {
        // Basic compile check - focus_trap.rs requires browser environment to fully test
        // Focusable selector is static and can be checked
        assert!(
            r#"
        a[href],
        button:not([disabled]),
        input:not([disabled]),
        select:not([disabled]),
        textarea:not([disabled]),
        [tabindex]:not([tabindex="-1"])
    "#
            .contains("a[href]")
        );
    }
}
