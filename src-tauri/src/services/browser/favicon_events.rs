use super::{favicon_runtime::FAVICONS, favicon_state::FaviconState, BrowserCommandError};
use serde::Serialize;
use ts_rs::TS;

pub(super) const FAVICON_EVENT: &str = "browser-favicons-changed-v1";
pub(super) const EVENT_VERSION: u8 = 1;

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrowserFaviconIcon {
    pub tab_id: String,
    pub png_base64: String,
}

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrowserFaviconSnapshot {
    pub event_version: u8,
    #[ts(type = "number")]
    pub revision: u64,
    pub conversation_id: String,
    pub icons: Vec<BrowserFaviconIcon>,
}

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
    FAVICONS
        .lock()
        .map(|state| snapshot(&state, conversation))
        .map_err(|_| BrowserCommandError::Internal)
}
