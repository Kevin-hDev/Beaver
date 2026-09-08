use super::BrowserCommandError;
#[cfg(any(test, native_browser))]
use super::{favicon_runtime::access, favicon_state::FaviconState};
use serde::Serialize;
#[cfg(test)]
use ts_rs::TS;

#[cfg(any(test, native_browser))]
pub(super) const FAVICON_EVENT: &str = "browser-favicons-changed-v1";
pub(super) const EVENT_VERSION: u8 = 1;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "camelCase")]
pub struct BrowserFaviconIcon {
    pub tab_id: String,
    pub png_base64: String,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "camelCase")]
pub struct BrowserFaviconSnapshot {
    pub event_version: u8,
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub conversation_id: String,
    pub icons: Vec<BrowserFaviconIcon>,
}

#[cfg(any(test, native_browser))]
pub(super) fn snapshot(state: &FaviconState, conversation: &str) -> BrowserFaviconSnapshot {
    BrowserFaviconSnapshot {
        event_version: EVENT_VERSION,
        revision: state.revision,
        conversation_id: conversation.to_owned(),
        icons: state
            .entries
            .iter()
            .filter(|entry| entry.key.session_id == conversation)
            .filter_map(|entry| {
                Some(BrowserFaviconIcon {
                    tab_id: entry.key.tab_id.clone(),
                    png_base64: entry.png.clone()?,
                })
            })
            .collect(),
    }
}

pub fn read_snapshot(conversation: &str) -> Result<BrowserFaviconSnapshot, BrowserCommandError> {
    crate::services::agent_local::session_store::validate_session_id(conversation)
        .map_err(|_| BrowserCommandError::InvalidInput)?;
    #[cfg(any(test, native_browser))]
    {
        Ok(access(None, |state| snapshot(state, conversation)))
    }
    #[cfg(not(any(test, native_browser)))]
    {
        // Linux has no CEF surface; keep the cross-platform IPC contract stable.
        Ok(BrowserFaviconSnapshot {
            event_version: EVENT_VERSION,
            revision: 0,
            conversation_id: conversation.to_owned(),
            icons: Vec::new(),
        })
    }
}
