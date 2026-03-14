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
            (&self.github, crate::icons::GITHUB, "GitHub"),
            (&self.discord, crate::icons::DISCORD, "Discord"),
            (&self.twitter, crate::icons::TWITTER, "Twitter"),
            (&self.mastodon, crate::icons::MASTODON, "Mastodon"),
        ];
        for (url, icon_svg, label) in links {
            if let Some(url) = url {
                let safe_url = crate::utils::html_escape(url);
                let icon = crate::icons::svg_icon(icon_svg, "20", "20", "");
                html.push_str(&format!(
                    r#"<a href="{}" class="oxidoc-social-link" target="_blank" rel="noopener noreferrer" aria-label="{}" title="{}">{}</a>"#,
                    safe_url, label, label, icon
                ));
            }
        }
        html
    }
}
