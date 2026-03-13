use super::ApiEndpoint;
use crate::utils::html_escape;
use std::fmt::Write;

fn highlight_code(code: &str, lang: &str) -> String {
    if oxidoc_highlight::is_supported(lang) {
        format!(
            "<pre><code class=\"language-{lang}\">{}</code></pre>",
            oxidoc_highlight::highlight(code, lang)
        )
    } else {
        format!(
            "<pre><code class=\"language-{lang}\">{}</code></pre>",
            html_escape(code)
        )
    }
}

/// Render an API endpoint as a two-column Scalar-style layout.
///
/// Left column: description, parameters, request body, responses
/// Right column: code snippets (tabbed), playground, response examples
pub fn render_endpoint_html(ep: &ApiEndpoint, base_url: Option<&str>) -> String {
    let mut html = String::with_capacity(4096);

    // Two-column wrapper
    html.push_str("<div class=\"oxidoc-api-page\">");

    // === LEFT COLUMN: Documentation ===
    html.push_str("<div class=\"oxidoc-api-docs\">");

    // Page title (summary or operationId or method+path)
    let fallback_title = format!("{} {}", ep.method, ep.path);
    let title = ep
        .summary
        .as_deref()
        .or(ep.operation_id.as_deref())
        .unwrap_or(&fallback_title);
    let _ = write!(html, "<h1>{}</h1>", html_escape(title));

    // Method badge + path
    let _ = write!(
        html,
        r#"<div class="oxidoc-api-method-line"><span class="oxidoc-api-method oxidoc-api-method-{}">{}</span> <code>{}</code></div>"#,
        html_escape(&ep.method.to_lowercase()),
        html_escape(&ep.method),
        html_escape(&ep.path)
    );

    if ep.deprecated {
        html.push_str(
            r#"<div class="oxidoc-callout oxidoc-callout-warning"><strong>Deprecated</strong>: This endpoint is deprecated and may be removed in a future version.</div>"#,
        );
    }

    if let Some(ref desc) = ep.description {
        let _ = write!(
            html,
            "<div class=\"oxidoc-api-description\">{}</div>",
            html_escape(desc)
        );
    }

    render_parameters(&ep.parameters, &mut html);
    render_request_body(&ep.request_body, &mut html);
    render_responses_docs(&ep.responses, &mut html);

    html.push_str("</div>"); // end oxidoc-api-docs

    // === RIGHT COLUMN: Code + Playground ===
    html.push_str("<div class=\"oxidoc-api-panel\">");
    html.push_str("<div class=\"oxidoc-api-panel-inner\">");

    // Static code snippets (tabbed)
    render_code_snippets(ep, base_url, &mut html);

    // API Playground island
    let mut props = serde_json::json!({
        "method": ep.method,
        "path": ep.path,
        "parameters": ep.parameters.iter().map(|p| {
            serde_json::json!({
                "name": p.name,
                "in": p.location,
                "type": p.schema_type,
                "required": p.required,
            })
        }).collect::<Vec<_>>(),
    });
    if let Some(url) = base_url {
        props["base_url"] = serde_json::Value::String(url.to_string());
    }
    let _ = write!(
        html,
        r##"<oxidoc-island data-island-type="api-playground" data-props='{}'></oxidoc-island>"##,
        serde_json::to_string(&props).unwrap_or_default()
    );

    // Response examples
    render_response_examples(&ep.responses, &mut html);

    html.push_str("</div>"); // end oxidoc-api-panel-inner
    html.push_str("</div>"); // end oxidoc-api-panel

    html.push_str("</div>"); // end oxidoc-api-page

    html
}

fn render_parameters(parameters: &[super::ApiParameter], html: &mut String) {
    if parameters.is_empty() {
        return;
    }

    let groups: &[(&str, &str)] = &[
        ("path", "Path Parameters"),
        ("query", "Query Parameters"),
        ("header", "Header Parameters"),
        ("cookie", "Cookie Parameters"),
    ];

    for &(location, title) in groups {
        let params: Vec<_> = parameters
            .iter()
            .filter(|p| p.location == location)
            .collect();
        if params.is_empty() {
            continue;
        }
        let _ = write!(html, "<h2>{title}</h2>");
        html.push_str(r#"<table class="oxidoc-api-field-table"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>"#);
        for param in &params {
            let badge = if param.required {
                r#" <span class="oxidoc-api-required">required</span>"#
            } else {
                r#" <span class="oxidoc-api-optional">?</span>"#
            };
            let desc = param
                .description
                .as_deref()
                .map(html_escape)
                .unwrap_or_default();
            let _ = write!(
                html,
                r#"<tr><td><code>{}</code>{}</td><td>{}</td><td>{}</td></tr>"#,
                html_escape(&param.name),
                badge,
                html_escape(&param.schema_type),
                desc,
            );
        }
        html.push_str("</tbody></table>");
    }
}

fn render_request_body(body: &Option<super::ApiRequestBody>, html: &mut String) {
    let Some(body) = body else { return };

    html.push_str(r#"<div class="oxidoc-api-body-header"><h2 id="request-body">Body</h2>"#);
    if body.required {
        html.push_str(r#"<span class="oxidoc-api-required">required</span>"#);
    }
    let _ = write!(
        html,
        r#"<span class="oxidoc-api-content-type">{}</span>"#,
        html_escape(&body.content_type)
    );
    html.push_str("</div>");
    if let Some(ref desc) = body.description {
        let _ = write!(html, "<p>{}</p>", html_escape(desc));
    }
    render_schema_field_table(&body.fields, html);
}

/// Render responses as tabbed panels with schema field tables.
fn render_responses_docs(responses: &[super::ApiResponse], html: &mut String) {
    if responses.is_empty() {
        return;
    }

    html.push_str("<h2 id=\"responses\">Responses</h2>");
    html.push_str(r#"<div class="oxidoc-api-responses">"#);

    // Tabs
    html.push_str(r#"<div class="oxidoc-api-tabs" role="tablist">"#);
    let mut first = true;
    for resp in responses {
        let status_class = match resp.status.chars().next() {
            Some('2') => "oxidoc-api-status-2xx",
            Some('3') => "oxidoc-api-status-3xx",
            Some('4') => "oxidoc-api-status-4xx",
            Some('5') => "oxidoc-api-status-5xx",
            _ => "",
        };
        let active = if first { " active" } else { "" };
        let selected = if first { "true" } else { "false" };
        let _ = write!(
            html,
            r#"<button class="oxidoc-api-tab{active}" data-tab="resp-{}" role="tab" aria-selected="{selected}"><span class="oxidoc-api-status {status_class}">{}</span></button>"#,
            html_escape(&resp.status),
            html_escape(&resp.status),
        );
        first = false;
    }
    html.push_str("</div>");

    // Tab panels
    let mut first = true;
    for resp in responses {
        let active = if first { " active" } else { "" };
        let _ = write!(
            html,
            r#"<div class="oxidoc-api-tab-panel{active}" data-panel="resp-{}">"#,
            html_escape(&resp.status),
        );

        // Description
        if !resp.description.is_empty() {
            let _ = write!(
                html,
                r#"<p class="oxidoc-api-response-desc">{}</p>"#,
                html_escape(&resp.description)
            );
        }

        // Schema type + content type (left-aligned)
        let mut meta_parts = Vec::new();
        if let Some(ref st) = resp.schema_type {
            meta_parts.push(format!(
                r#"<code class="oxidoc-api-schema-type">{}</code>"#,
                html_escape(st)
            ));
        }
        if let Some(ref ct) = resp.content_type {
            meta_parts.push(format!(
                r#"<span class="oxidoc-api-content-type">{}</span>"#,
                html_escape(ct)
            ));
        }
        if !meta_parts.is_empty() {
            let _ = write!(
                html,
                r#"<div class="oxidoc-api-response-meta">{}</div>"#,
                meta_parts.join(" ")
            );
        }

        // Field table
        render_schema_field_table(&resp.fields, html);

        html.push_str("</div>");
        first = false;
    }

    html.push_str("</div>"); // end oxidoc-api-responses
}

/// Render schema fields as a full-width table with nested field support.
fn render_schema_field_table(fields: &[super::SchemaField], html: &mut String) {
    if fields.is_empty() {
        return;
    }
    html.push_str(r#"<table class="oxidoc-api-field-table"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>"#);
    render_field_rows(fields, 0, html);
    html.push_str("</tbody></table>");
}

fn render_field_rows(fields: &[super::SchemaField], depth: usize, html: &mut String) {
    for field in fields {
        let indent = if depth > 0 {
            format!(r#" style="padding-left:{}rem""#, 0.75 + depth as f32 * 1.25)
        } else {
            String::new()
        };
        let badge = if field.required {
            r#" <span class="oxidoc-api-required">required</span>"#
        } else {
            r#" <span class="oxidoc-api-optional">?</span>"#
        };
        let desc = field
            .description
            .as_deref()
            .map(html_escape)
            .unwrap_or_default();
        let _ = write!(
            html,
            r#"<tr><td{indent}><code>{}</code>{}</td><td>{}</td><td>{}</td></tr>"#,
            html_escape(&field.name),
            badge,
            html_escape(&field.field_type),
            desc,
        );
        if !field.children.is_empty() {
            render_field_rows(&field.children, depth + 1, html);
        }
    }
}

/// Render static code snippets (curl, JS, Python) in a tabbed container.
fn render_code_snippets(ep: &ApiEndpoint, base_url: Option<&str>, html: &mut String) {
    let base = base_url.unwrap_or("https://api.example.com");
    let url = format!("{}{}", base, ep.path);

    // Generate snippets
    let curl = generate_curl(&ep.method, &url);
    let js = generate_js(&ep.method, &url);

    html.push_str(r#"<div class="oxidoc-api-snippets">"#);

    // Tabs
    html.push_str(r#"<div class="oxidoc-api-tabs" role="tablist">"#);
    html.push_str(r#"<button class="oxidoc-api-tab active" data-tab="curl" role="tab" aria-selected="true">cURL</button>"#);
    html.push_str(r#"<button class="oxidoc-api-tab" data-tab="js" role="tab" aria-selected="false">JavaScript</button>"#);
    html.push_str(r#"<button class="oxidoc-api-copy-btn" title="Copy code">Copy</button>"#);
    html.push_str("</div>");

    // Tab panels
    let _ = write!(
        html,
        r#"<div class="oxidoc-api-tab-panel active" data-panel="curl">{}</div>"#,
        highlight_code(&curl, "bash")
    );
    let _ = write!(
        html,
        r#"<div class="oxidoc-api-tab-panel" data-panel="js">{}</div>"#,
        highlight_code(&js, "javascript")
    );

    html.push_str("</div>"); // end oxidoc-api-snippets
}

/// Render response examples (right column) — no longer needed since fields are on the left.
fn render_response_examples(_responses: &[super::ApiResponse], _html: &mut String) {
    // Response schema is now rendered as field lists in the left column.
}

fn generate_curl(method: &str, url: &str) -> String {
    format!("curl -X {} '{}'", method, url)
}

fn generate_js(method: &str, url: &str) -> String {
    format!(
        "const response = await fetch('{}', {{\n  method: '{}',\n  headers: {{ 'Accept': 'application/json' }}\n}});\nconst data = await response.json();",
        url, method
    )
}

/// Render the API index page listing all endpoints grouped by tag.
pub fn render_api_index(endpoints: &[ApiEndpoint], spec_title: &str) -> String {
    use super::nav::endpoint_slug;
    use std::collections::BTreeMap;

    let mut html = String::with_capacity(2048);
    let _ = write!(html, "<h1>{}</h1>", html_escape(spec_title));

    let mut by_tag: BTreeMap<&str, Vec<&ApiEndpoint>> = BTreeMap::new();
    for ep in endpoints {
        let tag = ep.tags.first().map(|s| s.as_str()).unwrap_or("Untagged");
        by_tag.entry(tag).or_default().push(ep);
    }

    for (tag, eps) in &by_tag {
        let _ = write!(html, "<h2>{}</h2>", html_escape(tag));
        html.push_str(r#"<table class="oxidoc-api-params"><thead><tr><th>Method</th><th>Path</th><th>Description</th></tr></thead><tbody>"#);
        for ep in eps {
            let slug = endpoint_slug(ep);
            let desc = ep.summary.as_deref().unwrap_or("");
            let _ = write!(
                html,
                r#"<tr><td><span class="oxidoc-api-method oxidoc-api-method-{}">{}</span></td><td><a href="/{}">{}</a></td><td>{}</td></tr>"#,
                html_escape(&ep.method.to_lowercase()),
                html_escape(&ep.method),
                html_escape(&slug),
                html_escape(&ep.path),
                html_escape(desc),
            );
        }
        html.push_str("</tbody></table>");
    }

    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::parser::extract_endpoints;
    use crate::openapi::test_helpers::sample_spec;

    #[test]
    fn render_endpoint_contains_sections() {
        let spec = sample_spec();
        let endpoints = extract_endpoints(&spec);
        let get_pets = endpoints
            .iter()
            .find(|e| e.operation_id.as_deref() == Some("listPets"))
            .unwrap();
        let html = render_endpoint_html(get_pets, None);

        assert!(html.contains("GET"));
        assert!(html.contains("/pets"));
        assert!(html.contains("List all pets"));
        assert!(html.contains("Parameters"));
        assert!(html.contains("limit"));
        assert!(html.contains("api-playground"));
        assert!(html.contains("oxidoc-api-page"));
        assert!(html.contains("oxidoc-api-docs"));
        assert!(html.contains("oxidoc-api-panel"));
    }

    #[test]
    fn render_endpoint_has_code_snippets() {
        let spec = sample_spec();
        let endpoints = extract_endpoints(&spec);
        let get_pets = endpoints
            .iter()
            .find(|e| e.operation_id.as_deref() == Some("listPets"))
            .unwrap();
        let html = render_endpoint_html(get_pets, Some("https://api.example.com"));

        assert!(html.contains("curl"));
        assert!(html.contains("https://api.example.com/pets"));
        assert!(html.contains("oxidoc-api-snippets"));
    }
}
