use serde::{Deserialize, Serialize};

pub const MAX_BROWSER_TABS: usize = 10;
pub(super) const SESSION_VERSION: u8 = 2;
pub(super) const MAX_TITLE_CHARS: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct BrowserTabState {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub released: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct BrowserSessionState {
    pub tabs: Vec<BrowserTabState>,
    pub active_tab_id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum BrowserTabCreation {
    Created {
        session: BrowserSessionState,
    },
    ConfirmationRequired {
        candidate_id: String,
        candidate_title: String,
    },
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct PersistedBrowserSession {
    pub(super) version: u8,
    pub(super) state: PersistedBrowserSessionState,
    pub(super) recency: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PersistedBrowserSessionState {
    pub(super) tabs: Vec<PersistedBrowserTabState>,
    pub(super) active_tab_id: String,
    pub(super) generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct PersistedBrowserTabState {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) url: Option<String>,
}

#[derive(Default)]
#[cfg(any(test, browser_native_api))]
pub(super) struct BrowserRuntimeTabUpdate {
    pub(super) title: Option<String>,
    pub(super) url: Option<String>,
    pub(super) loading: Option<bool>,
    pub(super) can_go_back: Option<bool>,
    pub(super) can_go_forward: Option<bool>,
}

#[cfg(any(test, browser_native_api))]
pub(super) struct BrowserRuntimeUpdateResult {
    pub(super) changed: bool,
    pub(super) persisted_changed: bool,
}

pub(super) fn blank_tab(id: String) -> BrowserTabState {
    BrowserTabState {
        id,
        title: String::new(),
        url: None,
        loading: false,
        can_go_back: false,
        can_go_forward: false,
        released: false,
    }
}
