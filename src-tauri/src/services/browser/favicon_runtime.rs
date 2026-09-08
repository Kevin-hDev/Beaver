use super::{
    favicon_events::FAVICON_EVENT, favicon_state::FaviconState, favicon_store::FaviconStore,
};
use std::sync::LazyLock;
use tauri::Emitter;

// All readers and writers use this authority, including engine teardown.
static FAVICONS: LazyLock<FaviconStore> = LazyLock::new(FaviconStore::default);

pub(super) fn access<R>(
    app: Option<&tauri::AppHandle>,
    operation: impl FnOnce(&mut FaviconState) -> R,
) -> R {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        FAVICONS.access(app.is_some(), operation)
    }));
    let (result, snapshots) = match outcome {
        Ok((result, snapshots)) => (Ok(result), snapshots),
        Err(panic) => {
            // Repair and publish the empty state before the outer FFI guard handles
            // the panic. No later user action is needed to discover the corruption.
            let (_, snapshots) = FAVICONS.access(app.is_some(), |_| {});
            (Err(panic), snapshots)
        }
    };
    // No external callback while holding the state lock.
    if let Some(app) = app {
        for snapshot in snapshots {
            if app.emit(FAVICON_EVENT, snapshot).is_err() {
                log::debug!("[browser] favicon notification unavailable");
            }
        }
    }
    match result {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

pub(super) fn mutate(app: &tauri::AppHandle, operation: impl FnOnce(&mut FaviconState)) {
    access(Some(app), operation);
}
