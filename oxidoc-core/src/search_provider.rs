use crate::config::SearchConfig;
use crate::error::{OxidocError, Result};

/// Resolved search provider configuration.
#[derive(Debug, Clone)]
pub enum SearchProvider {
    /// Built-in Wasm search (lexical + semantic)
    Oxidoc { model_path: Option<String> },
    /// Algolia DocSearch
    Algolia {
        app_id: String,
        api_key: String,
        index_name: String,
    },
    /// Typesense InstantSearch
    Typesense {
        api_key: String,
        host: String,
        port: u16,
        protocol: String,
        collection_name: String,
    },
    /// Meilisearch
    Meilisearch {
        host: String,
        api_key: String,
        index_name: String,
    },
    /// Custom provider with raw script injection
    Custom {
        stylesheet: Option<String>,
        script: Option<String>,
        init_script: Option<String>,
    },
}

impl SearchProvider {
    /// Build a SearchProvider from configuration, validating all required fields.
    pub fn from_config(config: &SearchConfig) -> Result<Self> {
        match config.provider.as_str() {
            "oxidoc" => Ok(SearchProvider::Oxidoc {
                model_path: config.model_path.clone(),
            }),
            "algolia" => {
                let app_id =
                    config
                        .app_id
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "algolia".to_string(),
                            field: "app_id".to_string(),
                        })?;
                let api_key =
                    config
                        .api_key
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "algolia".to_string(),
                            field: "api_key".to_string(),
                        })?;
                let index_name = config.index_name.as_ref().ok_or_else(|| {
                    OxidocError::SearchProviderConfig {
                        provider: "algolia".to_string(),
                        field: "index_name".to_string(),
                    }
                })?;

                Ok(SearchProvider::Algolia {
                    app_id: app_id.clone(),
                    api_key: api_key.clone(),
                    index_name: index_name.clone(),
                })
            }
            "typesense" => {
                let api_key =
                    config
                        .api_key
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "typesense".to_string(),
                            field: "api_key".to_string(),
                        })?;
                let host =
                    config
                        .host
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "typesense".to_string(),
                            field: "host".to_string(),
                        })?;
                let port = config
                    .port
                    .ok_or_else(|| OxidocError::SearchProviderConfig {
                        provider: "typesense".to_string(),
                        field: "port".to_string(),
                    })?;
                let protocol =
                    config
                        .protocol
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "typesense".to_string(),
                            field: "protocol".to_string(),
                        })?;
                let collection_name = config.collection_name.as_ref().ok_or_else(|| {
                    OxidocError::SearchProviderConfig {
                        provider: "typesense".to_string(),
                        field: "collection_name".to_string(),
                    }
                })?;

                Ok(SearchProvider::Typesense {
                    api_key: api_key.clone(),
                    host: host.clone(),
                    port,
                    protocol: protocol.clone(),
                    collection_name: collection_name.clone(),
                })
            }
            "meilisearch" => {
                let host =
                    config
                        .host
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "meilisearch".to_string(),
                            field: "host".to_string(),
                        })?;
                let api_key =
                    config
                        .api_key
                        .as_ref()
                        .ok_or_else(|| OxidocError::SearchProviderConfig {
                            provider: "meilisearch".to_string(),
                            field: "api_key".to_string(),
                        })?;
                let index_name = config.index_name.as_ref().ok_or_else(|| {
                    OxidocError::SearchProviderConfig {
                        provider: "meilisearch".to_string(),
                        field: "index_name".to_string(),
                    }
                })?;

                Ok(SearchProvider::Meilisearch {
                    host: host.clone(),
                    api_key: api_key.clone(),
                    index_name: index_name.clone(),
                })
            }
            "custom" => Ok(SearchProvider::Custom {
                stylesheet: config.stylesheet.clone(),
                script: config.script.clone(),
                init_script: config.init_script.clone(),
            }),
            provider => {
                // Fall back to treating unknown providers as custom (for forward compatibility)
                tracing::warn!(
                    provider = provider,
                    "Unknown search provider; treating as custom"
                );
                Ok(SearchProvider::Custom {
                    stylesheet: None,
                    script: None,
                    init_script: None,
                })
            }
        }
    }

    /// Returns true only for the built-in Oxidoc provider.
    pub fn is_builtin(&self) -> bool {
        matches!(self, SearchProvider::Oxidoc { .. })
    }

    /// Render CSS <link> tags for <head>.
    pub fn render_head_tags(&self) -> String {
        match self {
            SearchProvider::Oxidoc { .. } => String::new(),
            SearchProvider::Algolia { .. } => {
                r#"    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@docsearch/css@3">"#
                    .to_string()
            }
            SearchProvider::Typesense { .. } => {
                r#"    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/typesense-docsearch-css@0.4">"#
                    .to_string()
            }
            SearchProvider::Meilisearch { .. } => {
                r#"    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@meilisearch/docs-searchbar.js@latest/dist/docs-searchbar.min.css">"#
                    .to_string()
            }
            SearchProvider::Custom {
                stylesheet: Some(css_url),
                ..
            } => {
                format!(r#"    <link rel="stylesheet" href="{}">"#, crate::utils::html_escape(css_url))
            }
            SearchProvider::Custom { .. } => String::new(),
        }
    }

    /// Render <script> tags for end of <body>.
    pub fn render_scripts(&self) -> String {
        match self {
            SearchProvider::Oxidoc { .. } => String::new(),
            SearchProvider::Algolia {
                app_id,
                api_key,
                index_name,
            } => {
                format!(
                    r#"    <script src="https://cdn.jsdelivr.net/npm/@docsearch/js@3"></script>
    <script>
docsearch({{
  appId: '{}',
  apiKey: '{}',
  indexName: '{}',
  container: '[data-oxidoc-search]'
}});
    </script>"#,
                    crate::utils::html_escape(app_id),
                    crate::utils::html_escape(api_key),
                    crate::utils::html_escape(index_name)
                )
            }
            SearchProvider::Typesense {
                api_key,
                host,
                port,
                protocol,
                collection_name,
            } => {
                format!(
                    r#"    <script src="https://cdn.jsdelivr.net/npm/typesense-docsearch.js@3/dist/cdn/typesense-docsearch.min.js"></script>
    <script>
typesenseDocsearch({{
  typesenseCollectionName: '{}',
  typesenseServerConfig: {{
    apiKey: '{}',
    nodes: [{{
      host: '{}',
      port: {},
      protocol: '{}'
    }}]
  }},
  container: '[data-oxidoc-search]'
}});
    </script>"#,
                    crate::utils::html_escape(collection_name),
                    crate::utils::html_escape(api_key),
                    crate::utils::html_escape(host),
                    port,
                    crate::utils::html_escape(protocol)
                )
            }
            SearchProvider::Meilisearch {
                host,
                api_key,
                index_name,
            } => {
                format!(
                    r#"    <script src="https://cdn.jsdelivr.net/npm/@meilisearch/docs-searchbar.js@latest"></script>
    <script>
docsSearchBar({{
  hostUrl: '{}',
  apiKey: '{}',
  indexUid: '{}',
  inputSelector: '[data-oxidoc-search]'
}});
    </script>"#,
                    crate::utils::html_escape(host),
                    crate::utils::html_escape(api_key),
                    crate::utils::html_escape(index_name)
                )
            }
            SearchProvider::Custom {
                script,
                init_script,
                ..
            } => {
                let mut result = String::new();
                if let Some(script_url) = script {
                    result.push_str(&format!(
                        r#"    <script src="{}"></script>"#,
                        crate::utils::html_escape(script_url)
                    ));
                    result.push('\n');
                }
                if let Some(init) = init_script {
                    // Safety: init_script is user-controlled config from oxidoc.toml,
                    // not external input. Users own their initialization code.
                    result.push_str(&format!(r#"    <script>{}</script>"#, init));
                }
                result
            }
        }
    }

    /// Render placeholder HTML for external providers.
    pub fn search_container_html(&self) -> String {
        match self {
            SearchProvider::Oxidoc { .. } => String::new(),
            _ => r#"<div data-oxidoc-search></div>"#.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oxidoc_provider_from_config() {
        let config = SearchConfig {
            model_path: Some("/path/to/model".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(provider.is_builtin());
        assert!(matches!(provider, SearchProvider::Oxidoc { .. }));
    }

    #[test]
    fn test_algolia_provider_from_config() {
        let config = SearchConfig {
            provider: "algolia".to_string(),
            app_id: Some("test-app-id".to_string()),
            api_key: Some("test-api-key".to_string()),
            index_name: Some("test-index".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(!provider.is_builtin());
        assert!(matches!(provider, SearchProvider::Algolia { .. }));
    }

    #[test]
    fn test_algolia_missing_required_field() {
        let config = SearchConfig {
            provider: "algolia".to_string(),
            app_id: Some("test-app-id".to_string()),
            index_name: Some("test-index".to_string()),
            ..SearchConfig::default()
        };
        let result = SearchProvider::from_config(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OxidocError::SearchProviderConfig { .. }
        ));
    }

    #[test]
    fn test_meilisearch_provider_from_config() {
        let config = SearchConfig {
            provider: "meilisearch".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: Some("docs".to_string()),
            host: Some("https://search.example.com".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(!provider.is_builtin());
    }

    #[test]
    fn test_custom_provider_from_config() {
        let config = SearchConfig {
            provider: "custom".to_string(),
            stylesheet: Some("https://cdn.example.com/search.css".to_string()),
            script: Some("https://cdn.example.com/search.js".to_string()),
            init_script: Some("SearchWidget.init({container: '#oxidoc-search-slot'})".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(!provider.is_builtin());
    }

    #[test]
    fn test_oxidoc_no_head_tags() {
        let provider = SearchProvider::from_config(&SearchConfig::default()).unwrap();
        assert_eq!(provider.render_head_tags(), "");
        assert_eq!(provider.render_scripts(), "");
        assert_eq!(provider.search_container_html(), "");
    }

    #[test]
    fn test_algolia_head_tags() {
        let config = SearchConfig {
            provider: "algolia".to_string(),
            app_id: Some("app".to_string()),
            api_key: Some("key".to_string()),
            index_name: Some("idx".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(provider.render_head_tags().contains("@docsearch/css@3"));
    }

    #[test]
    fn test_algolia_scripts() {
        let config = SearchConfig {
            provider: "algolia".to_string(),
            app_id: Some("test-app".to_string()),
            api_key: Some("test-key".to_string()),
            index_name: Some("test-index".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        let scripts = provider.render_scripts();
        assert!(scripts.contains("@docsearch/js@3"));
        assert!(scripts.contains("test-app"));
        assert!(scripts.contains("test-key"));
        assert!(scripts.contains("test-index"));
        assert!(scripts.contains("docsearch("));
    }

    #[test]
    fn test_custom_provider_renders_stylesheet() {
        let config = SearchConfig {
            provider: "custom".to_string(),
            stylesheet: Some("https://example.com/search.css".to_string()),
            ..SearchConfig::default()
        };
        let provider = SearchProvider::from_config(&config).unwrap();
        assert!(
            provider
                .render_head_tags()
                .contains("https://example.com/search.css")
        );
    }

    #[test]
    fn test_non_builtin_providers_have_search_container() {
        let configs = vec![
            SearchConfig {
                provider: "algolia".to_string(),
                app_id: Some("a".to_string()),
                api_key: Some("k".to_string()),
                index_name: Some("i".to_string()),
                ..SearchConfig::default()
            },
            SearchConfig {
                provider: "custom".to_string(),
                ..SearchConfig::default()
            },
        ];
        for config in configs {
            let provider = SearchProvider::from_config(&config).unwrap();
            assert!(!provider.search_container_html().is_empty());
            assert!(
                provider
                    .search_container_html()
                    .contains("data-oxidoc-search")
            );
        }
    }
}
