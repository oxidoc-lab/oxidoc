use crate::utils::html_escape;

/// A breadcrumb segment for page navigation.
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}

/// Generate breadcrumbs from a page slug.
///
/// Example: `"guides/setup"` → `[("Home", "/"), ("Guides", "/guides"), ("Setup", None)]`
pub fn generate_breadcrumbs(slug: &str) -> Vec<Breadcrumb> {
    let mut crumbs = vec![Breadcrumb {
        label: "Home".into(),
        href: Some("/".into()),
    }];

    let parts: Vec<&str> = slug.split('/').collect();
    let mut path = String::new();

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            path.push('/');
        }
        path.push_str(part);

        let label = humanize(part);
        let is_last = i == parts.len() - 1;

        crumbs.push(Breadcrumb {
            label,
            href: if is_last {
                None
            } else {
                Some(format!("/{path}"))
            },
        });
    }

    crumbs
}

/// Render breadcrumbs as an HTML `<nav>` element.
pub fn render_breadcrumbs(crumbs: &[Breadcrumb]) -> String {
    if crumbs.len() <= 1 {
        return String::new();
    }

    let mut html = String::from(r#"<nav class="oxidoc-breadcrumbs" aria-label="Breadcrumb"><ol>"#);

    for (i, crumb) in crumbs.iter().enumerate() {
        let is_last = i == crumbs.len() - 1;
        let safe_label = html_escape(&crumb.label);
        if let Some(href) = &crumb.href {
            html.push_str(&format!(
                r#"<li><a href="{}">{safe_label}</a></li>"#,
                html_escape(href)
            ));
        } else {
            html.push_str(&format!(r#"<li aria-current="page">{safe_label}</li>"#));
        }
        if !is_last {
            html.push_str(r#"<li class="separator" aria-hidden="true">/</li>"#);
        }
    }

    html.push_str("</ol></nav>");
    html
}

/// Convert a slug segment into a human-readable label.
fn humanize(segment: &str) -> String {
    // Strip leading numeric prefix (e.g., "01-intro" → "intro")
    let stripped = segment
        .find('-')
        .and_then(|pos| {
            if segment[..pos].chars().all(|c| c.is_ascii_digit()) {
                Some(&segment[pos + 1..])
            } else {
                None
            }
        })
        .unwrap_or(segment);

    stripped
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_breadcrumbs() {
        let crumbs = generate_breadcrumbs("intro");
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs[0].label, "Home");
        assert_eq!(crumbs[1].label, "Intro");
        assert!(crumbs[1].href.is_none());
    }

    #[test]
    fn nested_breadcrumbs() {
        let crumbs = generate_breadcrumbs("guides/01-quickstart");
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[0].label, "Home");
        assert_eq!(crumbs[1].label, "Guides");
        assert_eq!(crumbs[1].href.as_deref(), Some("/guides"));
        assert_eq!(crumbs[2].label, "Quickstart");
        assert!(crumbs[2].href.is_none());
    }

    #[test]
    fn render_breadcrumbs_html() {
        let crumbs = generate_breadcrumbs("guides/setup");
        let html = render_breadcrumbs(&crumbs);
        assert!(html.contains("Breadcrumb"));
        assert!(html.contains(r#"href="/""#));
        assert!(html.contains(r#"href="/guides""#));
        assert!(html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn single_segment_no_breadcrumb() {
        let crumbs = vec![Breadcrumb {
            label: "Home".into(),
            href: Some("/".into()),
        }];
        assert!(render_breadcrumbs(&crumbs).is_empty());
    }
}
