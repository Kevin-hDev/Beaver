use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) const VIEW_READY_EVENT: &str = "browser-view-ready-v1";
pub(super) const POPUP_REQUEST_EVENT: &str = "browser-popup-request-v1";
pub(super) const ENGINE_STOPPED_EVENT: &str = "browser-engine-stopped-v1";
pub(super) const BLOCKED_FEATURE_EVENT: &str = "browser-feature-blocked-v1";
pub(super) const TAB_STATE_CHANGED_EVENT: &str = "browser-tab-state-changed-v1";
pub(super) const EVENT_VERSION: u8 = 1;

static EVENT_GENERATION: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_event_generation() -> Option<u64> {
    EVENT_GENERATION
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            current.checked_add(1)
        })
        .ok()
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BrowserTabEvent {
    pub event_version: u8,
    pub generation: u64,
    pub conversation_id: String,
    pub tab_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PopupRequestEvent {
    pub event_version: u8,
    pub generation: u64,
    pub conversation_id: String,
    pub source_tab_id: String,
    pub url: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BrowserSessionEvent {
    pub event_version: u8,
    pub conversation_id: String,
    pub session: super::session_model::BrowserSessionState,
}
