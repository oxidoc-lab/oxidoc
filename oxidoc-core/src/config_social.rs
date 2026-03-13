use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct SocialConfig {
    #[serde(default)]
    pub github: Option<String>,
    #[serde(default)]
    pub discord: Option<String>,
    #[serde(default)]
    pub twitter: Option<String>,
    #[serde(default)]
    pub mastodon: Option<String>,
}

impl SocialConfig {
    /// Render social link icons for the header.
    pub fn render_header_icons(&self) -> String {
        let mut html = String::new();
        let links: &[(&Option<String>, &str, &str)] = &[
            (&self.github, "mdi:github", "GitHub"),
            (&self.discord, "ic:baseline-discord", "Discord"),
            (&self.twitter, "ri:twitter-x-fill", "Twitter"),
            (&self.mastodon, "ri:mastodon-fill", "Mastodon"),
        ];
        for (url, icon, label) in links {
            if let Some(url) = url {
                let safe_url = crate::utils::html_escape(url);
                html.push_str(&format!(
                    r#"<a href="{}" class="oxidoc-social-link" target="_blank" rel="noopener noreferrer" aria-label="{}" title="{}"><iconify-icon icon="{}" width="20" height="20"></iconify-icon></a>"#,
                    safe_url, label, label, icon
                ));
            }
        }
        html
    }
}
