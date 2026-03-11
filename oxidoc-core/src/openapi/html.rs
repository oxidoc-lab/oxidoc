use super::ApiEndpoint;
use crate::utils::html_escape;
use std::fmt::Write;

/// Render an API endpoint as an HTML page body.
pub fn render_endpoint_html(ep: &ApiEndpoint) -> String {
    let mut html = String::with_capacity(2048);

    // Title
    let _ = write!(
        html,
        "<h1><span class=\"oxidoc-api-method oxidoc-api-method-{}\">{}</span> <code>{}</code></h1>",
        html_escape(&ep.method.to_lowercase()),
        html_escape(&ep.method),
        html_escape(&ep.path)
    );

    if ep.deprecated {
        html.push_str(
            r#"<div class="oxidoc-callout oxidoc-callout-warning"><strong>Deprecated</strong>: This endpoint is deprecated and may be removed in a future version.</div>"#,
        );
    }

    // Summary / description
    if let Some(ref summary) = ep.summary {
        let _ = write!(
            html,
            "<p class=\"oxidoc-api-summary\">{}</p>",
            html_escape(summary)
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
    render_responses(&ep.responses, &mut html);

    // API Playground island placeholder
    let props = serde_json::json!({
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
    let _ = write!(
        html,
        r##"<oxidoc-island data-island-type="api-playground" data-props='{}'></oxidoc-island>"##,
        serde_json::to_string(&props).unwrap_or_default()
    );

    html
}

fn render_parameters(parameters: &[super::ApiParameter], html: &mut String) {
    if parameters.is_empty() {
        return;
    }

    html.push_str("<h2 id=\"parameters\">Parameters</h2>");
    html.push_str("<table class=\"oxidoc-api-params\"><thead><tr><th>Name</th><th>In</th><th>Type</th><th>Required</th><th>Description</th></tr></thead><tbody>");
    for param in parameters {
        let required = if param.required { "Yes" } else { "No" };
        let desc = param
            .description
            .as_deref()
            .map(html_escape)
            .unwrap_or_default();
        let _ = write!(
            html,
            "<tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
            html_escape(&param.name),
            html_escape(&param.location),
            html_escape(&param.schema_type),
            required,
            desc
        );
    }
    html.push_str("</tbody></table>");
}

fn render_request_body(body: &Option<super::ApiRequestBody>, html: &mut String) {
    let Some(body) = body else { return };

    html.push_str("<h2 id=\"request-body\">Request Body</h2>");
    let required = if body.required {
        " <em>(required)</em>"
    } else {
        ""
    };
    let _ = write!(
        html,
        "<p>Content-Type: <code>{}</code>{required}</p>",
        html_escape(&body.content_type)
    );
    if let Some(ref desc) = body.description {
        let _ = write!(html, "<p>{}</p>", html_escape(desc));
    }
    if !body.schema_json.is_empty() {
        let _ = write!(
            html,
            "<pre><code class=\"language-json\">{}</code></pre>",
            html_escape(&body.schema_json)
        );
    }
}

fn render_responses(responses: &[super::ApiResponse], html: &mut String) {
    if responses.is_empty() {
        return;
    }

    html.push_str("<h2 id=\"responses\">Responses</h2>");
    for resp in responses {
        let _ = write!(
            html,
            "<h3><code>{}</code></h3><p>{}</p>",
            html_escape(&resp.status),
            html_escape(&resp.description)
        );
        if let Some(ref ct) = resp.content_type {
            let _ = write!(
                html,
                "<p>Content-Type: <code>{}</code></p>",
                html_escape(ct)
            );
        }
        if let Some(ref schema) = resp.schema_json {
            let _ = write!(
                html,
                "<pre><code class=\"language-json\">{}</code></pre>",
                html_escape(schema)
            );
        }
    }
}

/// Generate a simple TOC for an API endpoint page.
pub fn render_api_toc(ep: &ApiEndpoint) -> String {
    let mut html = String::from(r#"<nav class="oxidoc-toc" aria-label="On this page"><ul>"#);

    if !ep.parameters.is_empty() {
        html.push_str(r##"<li class="toc-level-2"><a href="#parameters">Parameters</a></li>"##);
    }
    if ep.request_body.is_some() {
        html.push_str(r##"<li class="toc-level-2"><a href="#request-body">Request Body</a></li>"##);
    }
    if !ep.responses.is_empty() {
        html.push_str(r##"<li class="toc-level-2"><a href="#responses">Responses</a></li>"##);
    }

    html.push_str("</ul></nav>");
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
        let html = render_endpoint_html(get_pets);

        assert!(html.contains("GET"));
        assert!(html.contains("/pets"));
        assert!(html.contains("List all pets"));
        assert!(html.contains("Parameters"));
        assert!(html.contains("limit"));
        assert!(html.contains("Responses"));
        assert!(html.contains("api-playground"));
    }
}
