use std::sync::OnceLock;

use crate::api::ApiSourceFamily;

pub mod internal_network;
pub mod official_network;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScope {
    Read,
    Admin,
}

impl AuthScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub action: String,
    pub title: String,
    pub source: ApiSourceFamily,
    pub method: Option<String>,
    pub path: Option<String>,
    pub mutating: bool,
    pub auth_scope: AuthScope,
    pub verification_mode: Option<String>,
}

pub fn all_capabilities() -> &'static [Capability] {
    static ALL: OnceLock<Vec<Capability>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut caps = Vec::new();
        caps.extend(official_network::capabilities());
        caps.extend(internal_network::capabilities());
        caps
    })
}

pub fn find_capability(action: &str) -> Option<&'static Capability> {
    all_capabilities().iter().find(|cap| cap.action == action)
}
