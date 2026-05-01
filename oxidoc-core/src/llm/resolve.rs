use super::config::{LlmConfig, LlmPathOverride, ResolvedLlm};
use super::frontmatter::PageLlmFrontmatter;

/// Resolve effective LLM settings for a page using the
/// **site → path-prefix → page frontmatter** cascade.
///
/// Path overrides are applied in order of ascending segment length, so the
/// **longest matching prefix wins**. Frontmatter is applied last and overrides
/// everything.
pub fn resolve_llm_for_page(
    slug: &str,
    frontmatter: Option<&PageLlmFrontmatter>,
    config: &LlmConfig,
) -> ResolvedLlm {
    let mut resolved = ResolvedLlm {
        enabled: config.enabled,
        copy_button: config.copy_button,
    };

    let mut matching: Vec<&LlmPathOverride> = config
        .paths
        .iter()
        .filter(|p| path_is_prefix_of_slug(&p.path, slug))
        .collect();
    matching.sort_by_key(|p| segment_count(&p.path));

    for override_entry in matching {
        if let Some(v) = override_entry.enabled {
            resolved.enabled = v;
        }
        if let Some(v) = override_entry.copy_button {
            resolved.copy_button = v;
        }
    }

    if let Some(fm) = frontmatter {
        if let Some(v) = fm.enabled {
            resolved.enabled = v;
        }
        if let Some(v) = fm.copy_button {
            resolved.copy_button = v;
        }
    }

    resolved
}

/// Segment-aware prefix check: `"docs"` matches `"docs/intro"` but not `"docs-old/x"`.
fn path_is_prefix_of_slug(path: &str, slug: &str) -> bool {
    let path = path.trim_matches('/');
    if path.is_empty() {
        return true;
    }
    if slug == path {
        return true;
    }
    slug.starts_with(path) && slug.as_bytes().get(path.len()) == Some(&b'/')
}

fn segment_count(path: &str) -> usize {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> LlmConfig {
        LlmConfig::default()
    }

    fn override_disable(path: &str) -> LlmPathOverride {
        LlmPathOverride {
            path: path.into(),
            enabled: Some(false),
            copy_button: None,
        }
    }

    #[test]
    fn defaults_all_true() {
        let r = resolve_llm_for_page("intro", None, &cfg());
        assert!(r.enabled && r.copy_button);
    }

    #[test]
    fn site_disabled_propagates() {
        let mut c = cfg();
        c.enabled = false;
        assert!(!resolve_llm_for_page("intro", None, &c).enabled);
    }

    #[test]
    fn path_override_disables_section() {
        let mut c = cfg();
        c.paths.push(override_disable("guides"));
        assert!(!resolve_llm_for_page("guides/intro", None, &c).enabled);
        assert!(resolve_llm_for_page("docs/intro", None, &c).enabled);
    }

    #[test]
    fn path_segment_boundary() {
        let mut c = cfg();
        c.paths.push(override_disable("docs"));
        assert!(resolve_llm_for_page("docs-old/x", None, &c).enabled);
        assert!(!resolve_llm_for_page("docs/x", None, &c).enabled);
    }

    #[test]
    fn longest_prefix_wins() {
        let mut c = cfg();
        c.paths.push(LlmPathOverride {
            path: "docs".into(),
            enabled: Some(true),
            copy_button: None,
        });
        c.paths.push(override_disable("docs/internal"));
        assert!(resolve_llm_for_page("docs/public", None, &c).enabled);
        assert!(!resolve_llm_for_page("docs/internal/auth", None, &c).enabled);
    }

    #[test]
    fn frontmatter_overrides_path() {
        let mut c = cfg();
        c.paths.push(override_disable("docs"));
        let fm = PageLlmFrontmatter {
            enabled: Some(true),
            ..Default::default()
        };
        assert!(resolve_llm_for_page("docs/x", Some(&fm), &c).enabled);
    }

    #[test]
    fn frontmatter_partial_inherits_rest() {
        let mut c = cfg();
        c.copy_button = false;
        let fm = PageLlmFrontmatter {
            enabled: Some(true),
            ..Default::default()
        };
        let r = resolve_llm_for_page("intro", Some(&fm), &c);
        assert!(r.enabled);
        assert!(!r.copy_button);
    }
}
