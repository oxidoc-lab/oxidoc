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

    // Separate children into actions, tagline text, and extra components
    let mut actions: Vec<&rdx_ast::Node> = Vec::new();
    let mut tagline_nodes: Vec<&rdx_ast::Node> = Vec::new();
    let mut extra: Vec<&rdx_ast::Node> = Vec::new();
    for child in children {
        if matches!(child, rdx_ast::Node::Component(c) if c.name == "HeroAction") {
            actions.push(child);
        } else if matches!(child, rdx_ast::Node::Component(_)) {
            extra.push(child);
        } else {
            tagline_nodes.push(child);
        }
    }

    // Render tagline: prefer prop, fall back to body text
    if !tagline.is_empty() {
        let _ = write!(
            out,
            r#"<p class="oxidoc-hero-tagline">{}</p>"#,
            crate::utils::html_escape(tagline)
        );
    } else if !tagline_nodes.is_empty() {
        out.push_str(r#"<div class="oxidoc-hero-tagline">"#);
        for node in &tagline_nodes {
            crate::renderer::render_node(node, out, ctx);
        }
        out.push_str("</div>");
    }

    // Render action buttons
    if !actions.is_empty() {
        out.push_str(r#"<div class="oxidoc-hero-actions">"#);
        for node in &actions {
            crate::renderer::render_node(node, out, ctx);
        }
        out.push_str("</div>");
    }

    // Render extra components (e.g., install tabs) after buttons
    for node in &extra {
        crate::renderer::render_node(node, out, ctx);
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
    let kind = prop_str(props, "variant")
        .or_else(|| prop_str(props, "kind"))
        .unwrap_or("primary");
    let label = prop_str(props, "label");
    let _ = write!(
        out,
        r#"<a href="{}" class="oxidoc-hero-action oxidoc-hero-action-{kind}">"#,
        crate::utils::html_escape(href)
    );
    if let Some(label) = label {
        out.push_str(&crate::utils::html_escape(label));
    } else {
        render_children(children, out, ctx);
    }
    out.push_str("</a>");
}

pub(crate) fn render_static_feature_grid(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let extra_class = prop_str(props, "class");
    let class_attr = match extra_class {
        Some(c) => format!("oxidoc-feature-grid {}", crate::utils::html_escape(c)),
        None => "oxidoc-feature-grid".to_string(),
    };
    let _ = write!(out, r#"<div class="{class_attr}">"#);
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
    let extra_class = prop_str(props, "class");
    let class_attr = match extra_class {
        Some(c) => format!("oxidoc-feature {}", crate::utils::html_escape(c)),
        None => "oxidoc-feature".to_string(),
    };
    let _ = write!(out, r#"<div class="{class_attr}">"#);
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

/// Render a full-width section wrapper.
///
/// Props:
///   bg — background variant: "muted", "primary", "dark" (default: none)
///   id — optional HTML id attribute
///   padding — optional padding override (e.g. "4rem 0")
pub(crate) fn render_static_section(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let bg = prop_str(props, "bg");
    let id = prop_str(props, "id");
    let padding = prop_str(props, "padding");

    let extra_class = prop_str(props, "class");

    let mut classes = String::from("oxidoc-section");
    if let Some(bg) = bg {
        let _ = write!(classes, " oxidoc-section-{}", crate::utils::html_escape(bg));
    }
    if let Some(extra) = extra_class {
        let _ = write!(classes, " {}", crate::utils::html_escape(extra));
    }

    out.push_str(r#"<section class=""#);
    out.push_str(&classes);
    out.push('"');
    if let Some(id) = id {
        let _ = write!(out, r#" id="{}""#, crate::utils::html_escape(id));
    }
    if let Some(padding) = padding {
        let _ = write!(
            out,
            r#" style="padding:{}""#,
            crate::utils::html_escape(padding)
        );
    }
    out.push('>');
    out.push_str(r#"<div class="oxidoc-section-inner">"#);
    render_children(children, out, ctx);
    out.push_str("</div></section>");
}

/// Render a testimonial quote card.
///
/// Props:
///   author — author name
///   role   — author role/company
///   avatar — optional avatar image URL
pub(crate) fn render_static_testimonial(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let author = prop_str(props, "author").unwrap_or("");
    let role = prop_str(props, "role").unwrap_or("");
    let avatar = prop_str(props, "avatar");

    out.push_str(r#"<div class="oxidoc-testimonial">"#);
    out.push_str(r#"<div class="oxidoc-testimonial-quote">"#);
    out.push_str(r#"<span class="oxidoc-testimonial-open" aria-hidden="true">&ldquo;</span>"#);
    render_children(children, out, ctx);
    out.push_str(r#"<span class="oxidoc-testimonial-close" aria-hidden="true">&rdquo;</span>"#);
    out.push_str("</div>");
    out.push_str(r#"<div class="oxidoc-testimonial-author">"#);
    if let Some(avatar) = avatar {
        let _ = write!(
            out,
            r#"<img src="{}" alt="{}" class="oxidoc-testimonial-avatar" loading="lazy">"#,
            crate::utils::html_escape(avatar),
            crate::utils::html_escape(author),
        );
    }
    out.push_str(r#"<div class="oxidoc-testimonial-info">"#);
    if !author.is_empty() {
        let _ = write!(
            out,
            r#"<div class="oxidoc-testimonial-name">{}</div>"#,
            crate::utils::html_escape(author)
        );
    }
    if !role.is_empty() {
        let _ = write!(
            out,
            r#"<div class="oxidoc-testimonial-role">{}</div>"#,
            crate::utils::html_escape(role)
        );
    }
    out.push_str("</div></div></div>");
}

/// Render a grid wrapper for testimonials.
pub(crate) fn render_static_testimonial_grid(
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    out.push_str(r#"<div class="oxidoc-testimonial-grid">"#);
    render_children(children, out, ctx);
    out.push_str("</div>");
}

/// Render a responsive iframe embed container.
///
/// Props:
///   src         — iframe source URL
///   title       — iframe title (for accessibility)
///   aspectRatio — aspect ratio (default: "16/9")
pub(crate) fn render_static_embed(props: &HashMap<String, serde_json::Value>, out: &mut String) {
    let src = prop_str(props, "src").unwrap_or("");
    let title = prop_str(props, "title").unwrap_or("");
    let aspect_ratio = prop_str(props, "aspectRatio").unwrap_or("16/9");

    let _ = write!(
        out,
        r#"<div class="oxidoc-embed" style="aspect-ratio:{}">"#,
        crate::utils::html_escape(aspect_ratio)
    );
    let _ = write!(
        out,
        r#"<iframe src="{}" title="{}" loading="lazy" allowfullscreen></iframe>"#,
        crate::utils::html_escape(src),
        crate::utils::html_escape(title),
    );
    out.push_str("</div>");
}

/// Render a call-to-action block.
///
/// Props:
///   title       — CTA heading
///   description — CTA body text
pub(crate) fn render_static_cta(
    props: &HashMap<String, serde_json::Value>,
    children: &[rdx_ast::Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    let description = prop_str(props, "description").unwrap_or("");

    out.push_str(r#"<div class="oxidoc-cta">"#);
    if !title.is_empty() {
        let _ = write!(
            out,
            r#"<h2 class="oxidoc-cta-title">{}</h2>"#,
            crate::utils::html_escape(title)
        );
    }
    if !description.is_empty() {
        let _ = write!(
            out,
            r#"<p class="oxidoc-cta-description">{}</p>"#,
            crate::utils::html_escape(description)
        );
    }
    if !children.is_empty() {
        out.push_str(r#"<div class="oxidoc-cta-actions">"#);
        render_children(children, out, ctx);
        out.push_str("</div>");
    }
    out.push_str("</div>");
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
