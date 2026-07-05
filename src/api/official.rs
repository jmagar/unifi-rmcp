#[derive(Debug, Clone)]
pub struct OfficialNetworkApi {
    base_url: String,
}

impl OfficialNetworkApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn new_for_test(base_url: impl Into<String>) -> Self {
        Self::new(base_url)
    }

    pub fn path(&self, path: &str) -> String {
        let normalized = path.trim_start_matches('/');
        if let Some(rest) = normalized.strip_prefix("v1/") {
            format!("/proxy/network/integration/v1/{rest}")
        } else {
            format!("/proxy/network/integration/{normalized}")
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, self.path(path))
    }
}
