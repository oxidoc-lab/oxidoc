//! Authentication configuration for API requests.

use serde::{Deserialize, Serialize};

/// Authentication method supported by the API Playground.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum AuthType {
    /// No authentication
    #[serde(rename = "none")]
    #[default]
    None,
    /// API Key authentication with custom header name
    ApiKey {
        #[serde(default)]
        key: String,
        #[serde(default = "default_api_key_header")]
        header_name: String,
    },
    /// Bearer token authentication
    Bearer {
        #[serde(default)]
        token: String,
    },
    /// Basic authentication with username and password
    BasicAuth {
        #[serde(default)]
        username: String,
        #[serde(default)]
        password: String,
    },
}

fn default_api_key_header() -> String {
    "X-API-Key".into()
}

impl AuthType {
    /// Get the human-readable name of this auth type.
    pub fn name(&self) -> &'static str {
        match self {
            AuthType::None => "None",
            AuthType::ApiKey { .. } => "API Key",
            AuthType::Bearer { .. } => "Bearer Token",
            AuthType::BasicAuth { .. } => "Basic Auth",
        }
    }

    /// Apply this authentication to request headers.
    pub fn apply_to_headers(&self, headers: &mut Vec<(String, String)>) {
        match self {
            AuthType::None => {}
            AuthType::ApiKey { key, header_name } => {
                if !key.is_empty() {
                    headers.push((header_name.clone(), key.clone()));
                }
            }
            AuthType::Bearer { token } => {
                if !token.is_empty() {
                    headers.push(("Authorization".into(), format!("Bearer {}", token)));
                }
            }
            AuthType::BasicAuth { username, password } => {
                if !username.is_empty() {
                    let credentials = format!("{}:{}", username, password);
                    let encoded = base64_encode(&credentials);
                    headers.push(("Authorization".into(), format!("Basic {}", encoded)));
                }
            }
        }
    }
}

/// Simple base64 encoding for Basic Auth using JS's btoa.
fn base64_encode(input: &str) -> String {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = "btoa")]
        fn btoa_js(s: &str) -> String;
    }

    btoa_js(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_type_default() {
        assert_eq!(AuthType::default(), AuthType::None);
    }

    #[test]
    fn test_auth_type_names() {
        assert_eq!(AuthType::None.name(), "None");
        assert_eq!(
            AuthType::Bearer {
                token: "test".into()
            }
            .name(),
            "Bearer Token"
        );
        assert_eq!(
            AuthType::BasicAuth {
                username: "user".into(),
                password: "pass".into()
            }
            .name(),
            "Basic Auth"
        );
        assert_eq!(
            AuthType::ApiKey {
                key: "key".into(),
                header_name: "X-API-Key".into()
            }
            .name(),
            "API Key"
        );
    }

    #[test]
    fn test_apply_bearer_auth() {
        let auth = AuthType::Bearer {
            token: "mytoken".into(),
        };
        let mut headers = Vec::new();
        auth.apply_to_headers(&mut headers);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer mytoken");
    }

    #[test]
    fn test_apply_apikey_auth() {
        let auth = AuthType::ApiKey {
            key: "secret123".into(),
            header_name: "X-API-Key".into(),
        };
        let mut headers = Vec::new();
        auth.apply_to_headers(&mut headers);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "X-API-Key");
        assert_eq!(headers[0].1, "secret123");
    }

    #[test]
    fn test_apply_none_auth() {
        let auth = AuthType::None;
        let mut headers = Vec::new();
        auth.apply_to_headers(&mut headers);
        assert_eq!(headers.len(), 0);
    }
}
