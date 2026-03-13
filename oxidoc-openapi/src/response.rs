//! Response display component for API requests.

use crate::request::ApiResponseData;
use leptos::prelude::*;

/// Leptos view component for displaying API response data.
pub fn response_view(response: ApiResponseData) -> impl IntoView {
    let status_class = get_status_class(response.status);
    let formatted_body = format_response_body(&response.body);

    let duration_display = format!("{}ms", (response.duration_ms * 100.0).round() / 100.0);

    view! {
        <div class="api-response-section">
            <div class="api-response-header">
                <div class=format!("api-status {}", status_class)>
                    <span>{response.status}</span>
                    <span class="api-status-text">{response.status_text}</span>
                </div>
                <div class="api-duration">{duration_display}</div>
            </div>
            <div class="api-response-body">{formatted_body}</div>
        </div>
    }
}

/// Get the CSS class for a status code.
fn get_status_class(status: u16) -> &'static str {
    match status {
        200..=299 => "api-status-2xx",
        300..=399 => "api-status-3xx",
        400..=499 => "api-status-4xx",
        _ => "api-status-5xx",
    }
}

/// Format the response body for display (pretty-print JSON if applicable).
fn format_response_body(body: &str) -> String {
    // Try to parse as JSON and pretty-print
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        match serde_json::to_string_pretty(&value) {
            Ok(pretty) => pretty,
            Err(_) => body.to_string(),
        }
    } else {
        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_class_2xx() {
        assert_eq!(get_status_class(200), "api-status-2xx");
        assert_eq!(get_status_class(201), "api-status-2xx");
        assert_eq!(get_status_class(299), "api-status-2xx");
    }

    #[test]
    fn test_status_class_3xx() {
        assert_eq!(get_status_class(300), "api-status-3xx");
        assert_eq!(get_status_class(302), "api-status-3xx");
    }

    #[test]
    fn test_status_class_4xx() {
        assert_eq!(get_status_class(400), "api-status-4xx");
        assert_eq!(get_status_class(404), "api-status-4xx");
        assert_eq!(get_status_class(499), "api-status-4xx");
    }

    #[test]
    fn test_status_class_5xx() {
        assert_eq!(get_status_class(500), "api-status-5xx");
        assert_eq!(get_status_class(503), "api-status-5xx");
    }

    #[test]
    fn test_format_plain_text() {
        let plain = "Hello, world!";
        assert_eq!(format_response_body(plain), plain);
    }

    #[test]
    fn test_format_json() {
        let json = r#"{"name":"test","value":123}"#;
        let formatted = format_response_body(json);
        assert!(formatted.contains("name"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("value"));
        assert!(formatted.contains("123"));
    }
}
