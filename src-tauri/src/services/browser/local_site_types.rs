use serde::Serialize;

pub(super) const MAX_LOCAL_CANDIDATES: usize = 128;
pub(super) const MAX_LOCAL_RESULTS: usize = 32;
pub(super) const MAX_LOCAL_BODY_BYTES: usize = 64 * 1024;
pub(super) const MAX_LOCAL_TITLE_CHARS: usize = 80;
pub const LOCAL_SITES_CHANGED_EVENT: &str = "browser-local-sites-changed-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalSiteProtocol {
    Http,
    Https,
}

impl LocalSiteProtocol {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSite {
    pub url: String,
    pub title: String,
    pub port: u16,
    pub protocol: LocalSiteProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSiteScanResult {
    pub sites: Vec<LocalSite>,
    pub generation: u64,
    pub changed: bool,
}
