//! Static HTML rendering for landing-page components (Hero, Feature, Banner).

use std::collections::HashMap;
use std::fmt::Write;

use crate::renderer::{RenderCtx, render_children};
use crate::static_render::prop_str;

pub(crate) fn render_static_hero(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    let tagline = prop_str(props, "tagline").unwrap_or("");
    let image = prop_str(props, "image");
    out.push_str(r#"<section class="oxidoc-hero">"#);
    out.push_str(r#"<div class="oxidoc-hero-body">"#);
    if !title.is_empty() {
        let _ = write!(
            out,
            r#"<h1 class="oxidoc-hero-title">{}</h1>"#,
            crate::utils::html_escape(title)
        );
    }
    if !tagline.is_empty() {
        let _ = write!(
            out,
            r#"<p class="oxidoc-hero-tagline">{}</p>"#,
            crate::utils::html_escape(tagline)
        );
    }
    // Children render as action buttons / extra content
    let has_children = !children.is_empty();
    if has_children {
        out.push_str(r#"<div class="oxidoc-hero-actions">"#);
        render_children(children, out, ctx);
        out.push_str("</div>");
    }
    out.push_str("</div>");
    if let Some(img) = image {
        let _ = write!(
            out,
            r#"<div class="oxidoc-hero-image"><img src="{}" alt="" loading="lazy"></div>"#,
            crate::utils::html_escape(img)
        );
    }
    out.push_str("</section>");
}

pub(crate) fn render_static_hero_action(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let href = prop_str(props, "href").unwrap_or("#");
    let kind = prop_str(props, "kind").unwrap_or("primary");
    let _ = write!(
        out,
        r#"<a href="{}" class="oxidoc-hero-action oxidoc-hero-action-{kind}">"#,
        crate::utils::html_escape(href)
    );
    render_children(children, out, ctx);
    out.push_str("</a>");
}

pub(crate) fn render_static_feature_grid(
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    out.push_str(r#"<div class="oxidoc-feature-grid">"#);
    render_children(children, out, ctx);
    out.push_str("</div>");
}

pub(crate) fn render_static_feature(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    let icon = prop_str(props, "icon");
    out.push_str(r#"<div class="oxidoc-feature">"#);
    if let Some(icon) = icon {
        let _ = write!(
            out,
            r#"<div class="oxidoc-feature-icon">{}</div>"#,
            crate::utils::html_escape(icon)
        );
    }
    if !title.is_empty() {
        let _ = write!(
            out,
            r#"<h3 class="oxidoc-feature-title">{}</h3>"#,
            crate::utils::html_escape(title)
        );
    }
    out.push_str(r#"<div class="oxidoc-feature-desc">"#);
    render_children(children, out, ctx);
    out.push_str("</div></div>");
}

/// Render a dismissible announcement banner.
///
/// Props:
///   id          — unique identifier (default: "oxidoc-banner")
///   dismissible — show close button (default: true, also accepts "true"/"false" strings)
///   persist     — what happens after dismissing:
///                   "none"    — reappears on every page load (default)
///                   "session" — stays dismissed for the browser session (sessionStorage)
///                   "forever" — stays dismissed permanently (localStorage)
pub(crate) fn render_static_banner(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let id = prop_str(props, "id").unwrap_or("oxidoc-banner");
    let id_esc = crate::utils::html_escape(id);
    let dismissible = match props.get("dismissible") {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::String(s)) => s != "false",
        _ => true,
    };
    let persist = prop_str(props, "persist").unwrap_or("none");

    let _ = write!(
        out,
        r#"<div class="oxidoc-banner" id="{id_esc}" role="banner">"#,
    );
    out.push_str(r#"<div class="oxidoc-banner-content">"#);
    render_children(children, out, ctx);
    out.push_str("</div>");
    if dismissible {
        // Build onclick: always remove element, optionally persist dismissal
        let storage_js = match persist {
            "session" => format!(
                "try{{sessionStorage.setItem('oxidoc-banner-{id_esc}-dismissed','1')}}catch(e){{}}"
            ),
            "forever" => format!(
                "try{{localStorage.setItem('oxidoc-banner-{id_esc}-dismissed','1')}}catch(e){{}}"
            ),
            _ => String::new(), // "none" — no storage
        };
        let _ = write!(
            out,
            r#"<button class="oxidoc-banner-close" aria-label="Dismiss" onclick="this.parentElement.remove();{storage_js}">×</button>"#,
        );
    }
    out.push_str("</div>");

    // Script to hide on load if previously dismissed (only for session/forever)
    match persist {
        "session" => {
            let _ = write!(
                out,
                r#"<script>(function(){{try{{if(sessionStorage.getItem('oxidoc-banner-{id_esc}-dismissed')==='1'){{var b=document.getElementById('{id_esc}');if(b)b.remove()}}}}catch(e){{}}}})();</script>"#,
            );
        }
        "forever" => {
            let _ = write!(
                out,
                r#"<script>(function(){{try{{if(localStorage.getItem('oxidoc-banner-{id_esc}-dismissed')==='1'){{var b=document.getElementById('{id_esc}');if(b)b.remove()}}}}catch(e){{}}}})();</script>"#,
            );
        }
        _ => {} // "none" — no hide script
    }
}
