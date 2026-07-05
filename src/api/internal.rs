#[derive(Debug, Clone)]
pub struct InternalNetworkApi {
    base_url: String,
    site: String,
    legacy: bool,
}

impl InternalNetworkApi {
    pub fn new(base_url: impl Into<String>, site: impl Into<String>, legacy: bool) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            site: site.into(),
            legacy,
        }
    }

    pub fn new_for_test(
        base_url: impl Into<String>,
        site: impl Into<String>,
        legacy: bool,
    ) -> Self {
        Self::new(base_url, site, legacy)
    }

    pub fn v1_site_path(&self, suffix: &str) -> String {
        let suffix = suffix.trim_start_matches('/');
        let prefix = if self.legacy { "" } else { "/proxy/network" };
        format!("{prefix}/api/s/{site}/{suffix}", site = self.site)
    }

    pub fn v2_site_path(&self, suffix: &str) -> String {
        let suffix = suffix.trim_start_matches('/');
        if self.legacy {
            format!("/v2/api/site/{site}/{suffix}", site = self.site)
        } else {
            format!(
                "/proxy/network/v2/api/site/{site}/{suffix}",
                site = self.site
            )
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
