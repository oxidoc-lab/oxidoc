//! Keyboard navigation utilities for interactive components.
//!
//! Provides helpers to map keyboard events to standard actions for use in
//! dropdowns, tabs, menus, and other interactive components.

/// Standard keyboard actions for interactive components.
///
/// Maps common keyboard events to semantic actions that components can respond to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Activate/submit: Enter or Space
    Activate,
    /// Dismiss/close: Escape
    Dismiss,
    /// Navigate to next item: ArrowDown or ArrowRight
    Next,
    /// Navigate to previous item: ArrowUp or ArrowLeft
    Previous,
    /// Jump to first item: Home
    First,
    /// Jump to last item: End
    Last,
    /// No action mapped to this key
    None,
}

/// Classify a keyboard key into a standard action.
///
/// Maps raw key strings (from `KeyboardEvent.key`) to semantic actions.
///
/// # Key Mappings
/// * `"Enter"` or `" "` (space) → `Activate`
/// * `"Escape"` → `Dismiss`
/// * `"ArrowDown"` or `"ArrowRight"` → `Next`
/// * `"ArrowUp"` or `"ArrowLeft"` → `Previous`
/// * `"Home"` → `First`
/// * `"End"` → `Last`
/// * All other keys → `None`
///
/// # Arguments
/// * `key` - The key string from a KeyboardEvent (case-sensitive)
///
/// # Example
/// ```ignore
/// use oxidoc_island::keyboard::{classify_key, KeyAction};
///
/// assert_eq!(classify_key("Enter"), KeyAction::Activate);
/// assert_eq!(classify_key("Escape"), KeyAction::Dismiss);
/// assert_eq!(classify_key("ArrowDown"), KeyAction::Next);
/// assert_eq!(classify_key("Unknown"), KeyAction::None);
/// ```
pub fn classify_key(key: &str) -> KeyAction {
    match key {
        "Enter" | " " => KeyAction::Activate,
        "Escape" => KeyAction::Dismiss,
        "ArrowDown" | "ArrowRight" => KeyAction::Next,
        "ArrowUp" | "ArrowLeft" => KeyAction::Previous,
        "Home" => KeyAction::First,
        "End" => KeyAction::Last,
        _ => KeyAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_activate_keys() {
        assert_eq!(classify_key("Enter"), KeyAction::Activate);
        assert_eq!(classify_key(" "), KeyAction::Activate);
    }

    #[test]
    fn classify_dismiss_key() {
        assert_eq!(classify_key("Escape"), KeyAction::Dismiss);
    }

    #[test]
    fn classify_next_keys() {
        assert_eq!(classify_key("ArrowDown"), KeyAction::Next);
        assert_eq!(classify_key("ArrowRight"), KeyAction::Next);
    }

    #[test]
    fn classify_previous_keys() {
        assert_eq!(classify_key("ArrowUp"), KeyAction::Previous);
        assert_eq!(classify_key("ArrowLeft"), KeyAction::Previous);
    }

    #[test]
    fn classify_first_key() {
        assert_eq!(classify_key("Home"), KeyAction::First);
    }

    #[test]
    fn classify_last_key() {
        assert_eq!(classify_key("End"), KeyAction::Last);
    }

    #[test]
    fn classify_unknown_keys() {
        assert_eq!(classify_key("Unknown"), KeyAction::None);
        assert_eq!(classify_key("a"), KeyAction::None);
        assert_eq!(classify_key("1"), KeyAction::None);
        assert_eq!(classify_key("Meta"), KeyAction::None);
        assert_eq!(classify_key(""), KeyAction::None);
    }

    #[test]
    fn key_action_derives() {
        let action = KeyAction::Activate;
        let cloned = action;
        assert_eq!(action, cloned);
        assert_eq!(format!("{:?}", action), "Activate");
    }

    #[test]
    fn classify_key_case_sensitive() {
        // Keys from KeyboardEvent.key are case-sensitive
        assert_eq!(classify_key("enter"), KeyAction::None);
        assert_eq!(classify_key("ENTER"), KeyAction::None);
        assert_eq!(classify_key("escape"), KeyAction::None);
        assert_eq!(classify_key("ESCAPE"), KeyAction::None);
    }
}
