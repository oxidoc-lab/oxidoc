//! API request execution using the Fetch API.

use oxidoc_island::IslandError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// A prepared API request ready to be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
}

impl ApiRequest {
    /// Create a new API request.
    pub fn new(method: String, url: String) -> Self {
        Self {
            method,
            url,
            headers: Vec::new(),
            body: None,
        }
    }

    /// Add a header to the request.
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    /// Set the request body.
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
}

/// Response data returned from an API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponseData {
    pub status: u16,
    pub status_text: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: f64,
}

/// Execute an API request and return the response data.
pub async fn execute_request(request: ApiRequest) -> Result<ApiResponseData, IslandError> {
    let start = js_sys::Date::now();

    // Build fetch request options
    let init = web_sys::RequestInit::new();
    init.set_method(&request.method);
    init.set_mode(web_sys::RequestMode::Cors);

    // Add headers
    let headers = web_sys::Headers::new().map_err(|e| IslandError {
        message: format!("Failed to create headers: {e:?}"),
    })?;

    for (name, value) in &request.headers {
        headers.append(name, value).map_err(|e| IslandError {
            message: format!("Failed to append header {}: {e:?}", name),
        })?;
    }

    init.set_headers(&headers);

    // Add body if present
    if let Some(ref body) = request.body {
        let body_js = JsValue::from_str(body);
        init.set_body(&body_js);
    }

    // Execute fetch
    let window = web_sys::window().ok_or_else(|| IslandError {
        message: "Window object not available".into(),
    })?;

    let request_obj =
        web_sys::Request::new_with_str_and_init(&request.url, &init).map_err(|e| IslandError {
            message: format!("Failed to create request: {e:?}"),
        })?;

    let response_promise = window.fetch_with_request(&request_obj);
    let response = JsFuture::from(response_promise)
        .await
        .map_err(|e| IslandError {
            message: format!("Fetch failed: {e:?}"),
        })?;

    let response: web_sys::Response = response.dyn_into().map_err(|e| IslandError {
        message: format!("Response conversion failed: {e:?}"),
    })?;

    let status = response.status();
    let status_text = response.status_text();

    // Extract response headers
    let mut response_headers = Vec::new();
    let headers = response.headers();

    // Note: Headers are not easily iterable in web-sys, so we capture common ones
    if let Ok(Some(content_type)) = headers.get("content-type") {
        response_headers.push(("content-type".into(), content_type));
    }
    if let Ok(Some(content_length)) = headers.get("content-length") {
        response_headers.push(("content-length".into(), content_length));
    }

    // Extract body as text
    let body_promise = response.text().map_err(|e| IslandError {
        message: format!("Failed to read response body: {e:?}"),
    })?;

    let body_value: JsValue = JsFuture::from(body_promise)
        .await
        .map_err(|e| IslandError {
            message: format!("Failed to extract body text: {e:?}"),
        })?;

    let body = body_value.as_string().ok_or_else(|| IslandError {
        message: "Response body is not a string".into(),
    })?;

    let duration_ms = js_sys::Date::now() - start;

    Ok(ApiResponseData {
        status,
        status_text,
        headers: response_headers,
        body,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_request_creation() {
        let req = ApiRequest::new("GET".into(), "https://api.example.com/users".into());
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/users");
        assert!(req.headers.is_empty());
        assert!(req.body.is_none());
    }

    #[test]
    fn test_api_request_with_headers() {
        let req = ApiRequest::new("GET".into(), "https://api.example.com".into())
            .with_header("Authorization".into(), "Bearer token".into())
            .with_header("X-Custom".into(), "value".into());

        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0].0, "Authorization");
        assert_eq!(req.headers[1].1, "value");
    }

    #[test]
    fn test_api_request_with_body() {
        let req = ApiRequest::new("POST".into(), "https://api.example.com".into())
            .with_body("{\"name\": \"test\"}".into());

        assert_eq!(req.body, Some("{\"name\": \"test\"}".into()));
    }
}
