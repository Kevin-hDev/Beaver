use super::{
    browser_events::{
        next_event_generation, BrowserTabEvent, BLOCKED_FEATURE_EVENT, EVENT_VERSION,
    },
    browser_view_key::BrowserViewKey,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::Emitter;

#[derive(Clone)]
pub(super) struct BlockedFeatureNotifier {
    app: tauri::AppHandle,
    key: BrowserViewKey,
    emitted: Arc<AtomicBool>,
}

impl BlockedFeatureNotifier {
    pub(super) fn new(app: tauri::AppHandle, key: BrowserViewKey) -> Self {
        Self {
            app,
            key,
            emitted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn publish_once(&self) {
        if self.emitted.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(generation) = next_event_generation() else {
            return;
        };
        let _ = self.app.emit(
            BLOCKED_FEATURE_EVENT,
            BrowserTabEvent {
                event_version: EVENT_VERSION,
                generation,
                conversation_id: self.key.session_id.clone(),
                tab_id: self.key.tab_id.clone(),
            },
        );
    }
}
