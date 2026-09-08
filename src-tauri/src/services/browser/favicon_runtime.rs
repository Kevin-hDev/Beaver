use super::favicon_state::FaviconState;
use std::sync::{LazyLock, Mutex};

// One process-wide authority; no favicon enters session persistence.
pub(super) static FAVICONS: LazyLock<Mutex<FaviconState>> = LazyLock::new(Mutex::default);

pub(super) fn mutate(app: &tauri::AppHandle, operation: impl FnOnce(&mut FaviconState)) {
    let _ = app;
    match FAVICONS.lock() {
        Ok(mut state) => operation(&mut state),
        Err(_) => log::warn!("[browser] favicon state unavailable"),
    }
}
