use super::{
    favicon_events::{snapshot, FAVICON_EVENT},
    favicon_state::FaviconState,
};
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

// One process-wide authority; no favicon enters session persistence.
pub(super) static FAVICONS: LazyLock<Mutex<FaviconState>> = LazyLock::new(Mutex::default);

pub(super) fn mutate(app: &tauri::AppHandle, operation: impl FnOnce(&mut FaviconState)) {
    let snapshots = match FAVICONS.lock() {
        Ok(mut state) => {
            // At most twice the bounded cache size; include evicted conversations
            // so their mounted panels also receive an empty/reduced snapshot.
            let mut conversations: Vec<String> = state
                .entries
                .iter()
                .map(|entry| entry.key.session_id.clone())
                .collect();
            let revision = state.revision;
            operation(&mut state);
            if revision == state.revision {
                return;
            }
            conversations.extend(
                state
                    .entries
                    .iter()
                    .map(|entry| entry.key.session_id.clone()),
            );
            conversations.sort();
            conversations.dedup();
            conversations
                .iter()
                .map(|id| snapshot(&state, id))
                .collect::<Vec<_>>()
        }
        Err(_) => {
            log::warn!("[browser] favicon state unavailable");
            return;
        }
    };
    // Never invoke an external callback while holding the state lock.
    for snapshot in snapshots {
        if app.emit(FAVICON_EVENT, snapshot).is_err() {
            log::debug!("[browser] favicon notification unavailable");
        }
    }
}
