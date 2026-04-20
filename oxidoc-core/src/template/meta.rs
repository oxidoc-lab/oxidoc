/// Collect all `property:value` and `name:value` keys from a `<meta>` HTML fragment.
pub(crate) fn collect_meta_keys(fragment: &str) -> std::collections::HashSet<String> {
    let mut keys = std::collections::HashSet::new();
    for attr in ["property", "name"] {
        let marker = format!("{attr}=\"");
        let mut rest = fragment;
        while let Some(rel) = rest.find(&marker) {
            let after = &rest[rel + marker.len()..];
            if let Some(end) = after.find('"') {
                keys.insert(format!("{}:{}", attr, &after[..end]));
                rest = &after[end + 1..];
            } else {
                break;
            }
        }
    }
    keys
}

/// Remove default `<meta>` tags from `html` that are overridden by tags in `extra_head`.
pub(crate) fn remove_overridden_meta_tags(html: String, extra_head: &str) -> String {
    let overridden = collect_meta_keys(extra_head);
    if overridden.is_empty() {
        return html;
    }
    let mut result = html;
    for key in &overridden {
        if let Some((attr, val)) = key.split_once(':') {
            let needle = format!("<meta {attr}=\"{val}\"");
            if let Some(start) = result.find(&needle) {
                if let Some(rel_end) = result[start..].find('>') {
                    let end = start + rel_end + 1;
                    let remove_end = if result.as_bytes().get(end) == Some(&b'\n') {
                        end + 1
                    } else {
                        end
                    };
                    result.drain(start..remove_end);
                }
            }
        }
    }
    result
}
