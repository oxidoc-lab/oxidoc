//! Leptos view for the API Playground component.

use super::types::ApiPlaygroundProps;
use crate::auth::AuthType;
use crate::codegen::CodegenRequest;
use crate::request::{ApiResponseData, execute_request};
use crate::response::response_view;
use leptos::prelude::*;

pub fn playground_view(props: ApiPlaygroundProps) -> impl IntoView {
    // State signals
    #[allow(deprecated)]
    let (loading, set_loading) = create_signal(false);
    #[allow(deprecated)]
    let (response, set_response) = create_signal::<Option<ApiResponseData>>(None);
    #[allow(deprecated)]
    let (error, set_error) = create_signal::<Option<String>>(None);

    // Parameter input signals
    #[allow(deprecated)]
    let (param_values, set_param_values) = create_signal::<Vec<(String, String)>>(
        props
            .parameters
            .iter()
            .map(|p| (p.name.clone(), String::new()))
            .collect(),
    );

    // Auth state
    #[allow(deprecated)]
    let (auth_type, set_auth_type) = create_signal(AuthType::None);
    #[allow(deprecated)]
    let (auth_expanded, set_auth_expanded) = create_signal(false);

    // Request body
    #[allow(deprecated)]
    let (request_body, set_request_body) = create_signal(String::new());

    // Base URL
    let base_url_computed = props.base_url.clone().unwrap_or_else(|| {
        web_sys::window()
            .map(|w| w.location())
            .and_then(|l| l.origin().ok())
            .unwrap_or_else(|| "".into())
    });

    let base_url_for_send = base_url_computed.clone();
    let base_url_for_response = base_url_computed.clone();
    let props_method = props.method.clone();
    let props_method_response = props.method.clone();
    let props_path = props.path.clone();
    let props_path_response = props.path.clone();
    let has_parameters = !props.parameters.is_empty();
    let has_request_body = props.request_body_schema.is_some();

    let handle_send_request = move |_| {
        if loading.get() {
            return;
        }

        set_loading.set(true);
        set_error.set(None);
        set_response.set(None);

        let base_url_clone = base_url_for_send.clone();
        let method = props_method.clone();
        let path = props_path.clone();
        let params = param_values.get();
        let auth = auth_type.get();
        let body = request_body.get();

        wasm_bindgen_futures::spawn_local(async move {
            let url = build_url(&base_url_clone, &path, &params);
            let mut headers = Vec::new();
            auth.apply_to_headers(&mut headers);

            if !body.is_empty() && !headers.iter().any(|(k, _)| k == "Content-Type") {
                headers.push(("Content-Type".into(), "application/json".into()));
            }

            let request = crate::request::ApiRequest::new(method.clone(), url)
                .with_header("Accept".into(), "application/json".into());
            let request = headers
                .into_iter()
                .fold(request, |r, (k, v)| r.with_header(k, v));
            let request = if !body.is_empty() {
                request.with_body(body)
            } else {
                request
            };

            match execute_request(request).await {
                Ok(resp) => {
                    set_response.set(Some(resp));
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e.message.clone()));
                }
            }
            set_loading.set(false);
        });
    };

    let method_class = format!("api-method api-method-{}", props.method.to_lowercase());

    view! {
        <div class="api-header">
            <span class=method_class>{props.method.clone()}</span>
            <code class="api-path">{props.path.clone()}</code>
        </div>

        // Parameters section
        <Show when=move || has_parameters>
            <div class="api-section">
                <div class="api-section-title">
                    <span>"Parameters"</span>
                </div>
                <div class="api-param-group">
                    {props.parameters.iter().map(|param| {
                        let param_name = param.name.clone();
                        let param_location = param.location.clone();
                        let param_type = param.param_type.clone();
                        let param_required = param.required;

                        view! {
                            <div class="api-param-field">
                                <label class="api-param-label">
                                    <span>{param.name.clone()}</span>
                                    <Show when=move || param_required>
                                        <span class="api-param-required">"*"</span>
                                    </Show>
                                    <span class="api-param-type">
                                        "(" {param_location} " - " {param_type} ")"
                                    </span>
                                </label>
                                <input
                                    type="text"
                                    class="api-input"
                                    placeholder=format!("Enter {}", param_name)
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_param_values.update(|params| {
                                            if let Some(p) = params.iter_mut().find(|(n, _)| n == &param_name) {
                                                p.1 = value;
                                            }
                                        });
                                    }
                                />
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </Show>

        // Authentication section
        <div class="api-section">
            <div
                class="api-section-title"
                role="button"
                tabindex=0
                on:click=move |_| set_auth_expanded.update(|v| *v = !*v)
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        set_auth_expanded.update(|v| *v = !*v);
                    }
                }
            >
                <span class="api-section-toggle" class:collapsed=move || !auth_expanded.get()>
                    "▼"
                </span>
                <span>"Authentication"</span>
            </div>
            <Show when=move || auth_expanded.get()>
                <div class="api-auth-inputs">
                    <div class="api-param-field">
                        <label class="api-param-label">"Auth Type"</label>
                        <select
                            class="api-select"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let new_auth = match value.as_str() {
                                    "apikey" => AuthType::ApiKey {
                                        key: String::new(),
                                        header_name: "X-API-Key".into(),
                                    },
                                    "bearer" => AuthType::Bearer {
                                        token: String::new(),
                                    },
                                    "basic" => AuthType::BasicAuth {
                                        username: String::new(),
                                        password: String::new(),
                                    },
                                    _ => AuthType::None,
                                };
                                set_auth_type.set(new_auth);
                            }
                        >
                            <option value="none">"None"</option>
                            <option value="apikey">"API Key"</option>
                            <option value="bearer">"Bearer Token"</option>
                            <option value="basic">"Basic Auth"</option>
                        </select>
                    </div>
                    <p style="opacity: 0.6">"Configure authentication details above"</p>
                </div>
            </Show>
        </div>

        // Request body section
        <Show when=move || has_request_body>
            <div class="api-section">
                <label class="api-param-label">"Request Body (JSON)"</label>
                <textarea
                    class="api-textarea"
                    placeholder="Enter JSON request body"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        set_request_body.set(value);
                    }
                />
            </div>
        </Show>

        // Send button
        <div class="api-section">
            <button
                class="api-button api-button-primary"
                disabled=move || loading.get()
                on:click=handle_send_request
            >
                <Show when=move || loading.get() fallback=|| "Send Request">
                    <span class="api-loading-spinner"></span>
                    "Sending..."
                </Show>
            </button>
        </div>

        // Error display
        {
            move || error.get().map(|err| {
                view! {
                    <div class="api-error-message">
                        <strong>"Error: "</strong>
                        {err}
                    </div>
                }
            })
        }

        // Response display
        {
            move || response.get().map(|resp| {
                let method = props_method_response.clone();
                let url = build_url(&base_url_for_response, &props_path_response, &param_values.get());
                let mut headers = Vec::new();
                auth_type.get().apply_to_headers(&mut headers);

                let codegen_request = CodegenRequest {
                    method,
                    url,
                    headers,
                    body: if request_body.get().is_empty() {
                        None
                    } else {
                        Some(request_body.get())
                    },
                };

                let code_snippet = codegen_request.to_curl();
                let copy_callback = Callback::new(|_| {});

                view! {
                    <>
                        {response_view(resp, code_snippet, copy_callback)}
                    </>
                }
            })
        }
    }
}

/// Build the full URL from base URL, path, and parameter values.
fn build_url(base_url: &str, path: &str, params: &[(String, String)]) -> String {
    let mut url = format!("{}{}", base_url, path);

    // Replace path parameters
    for (name, value) in params {
        let placeholder = format!("{{{}}}", name);
        url = url.replace(&placeholder, value);
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_no_params() {
        let url = build_url("https://api.example.com", "/users", &[]);
        assert_eq!(url, "https://api.example.com/users");
    }

    #[test]
    fn test_build_url_with_params() {
        let url = build_url(
            "https://api.example.com",
            "/users/{id}/posts/{postId}",
            &[("id".into(), "42".into()), ("postId".into(), "1".into())],
        );
        assert_eq!(url, "https://api.example.com/users/42/posts/1");
    }
}
