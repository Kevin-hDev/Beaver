mod store;
pub mod types;
pub(crate) mod window;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

const CHANGED_EVENT: &str = "update-operation-changed";
const DISMISSED_EVENT: &str = "update-operation-dismissed";

#[derive(Clone, Default)]
pub struct UpdateProgressRuntime {
    store: Arc<Mutex<store::UpdateProgressStore>>,
    #[cfg(not(target_os = "linux"))]
    position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
}

impl UpdateProgressRuntime {
    pub fn upsert(
        &self,
        app: &AppHandle,
        mut operation: UpdateOperationSnapshot,
    ) -> Result<bool, String> {
        let mut store = self.store.lock().map_err(|_| public_error())?;
        resolve_cancelling_terminal(store.get(&operation.id), &mut operation);
        let should_show = should_show_window(store.get(&operation.id), &operation);
        operation.sequence = match store.get(&operation.id) {
            Some(current) => {
                let mut previous = current.clone();
                previous.sequence = operation.sequence;
                if previous == operation {
                    return Ok(false);
                }
                current
                    .sequence
                    .checked_add(1)
                    .ok_or_else(|| "update-progress-invalid".to_string())?
            }
            None => 1,
        };
        let changed = store.upsert(operation.clone()).map_err(str::to_string)?;
        drop(store);
        if changed {
            app.emit_to(window::WINDOW_LABEL, CHANGED_EVENT, &operation)
                .map_err(|_| public_error())?;
            if should_show {
                self.show_window(app)?;
            }
        }
        Ok(changed)
    }

    pub fn finish(
        &self,
        app: &AppHandle,
        operation: UpdateOperationSnapshot,
    ) -> Result<bool, String> {
        if !operation.status.is_terminal() {
            return Err("update-progress-invalid".to_string());
        }
        self.upsert(app, operation)
    }

    pub fn dismiss(&self, app: &AppHandle, id: &str) -> Result<bool, String> {
        let removed = self
            .store
            .lock()
            .map_err(|_| public_error())?
            .dismiss(id)
            .map_err(str::to_string)?;
        if removed {
            app.emit_to(window::WINDOW_LABEL, DISMISSED_EVENT, id)
                .map_err(|_| public_error())?;
            if self.snapshot()?.is_empty() {
                if let Some(window) = app.get_webview_window(window::WINDOW_LABEL) {
                    window.close().map_err(|_| public_error())?;
                }
            }
        }
        Ok(removed)
    }

    pub fn snapshot(&self) -> Result<Vec<UpdateOperationSnapshot>, String> {
        self.store
            .lock()
            .map(|store| store.snapshot())
            .map_err(|_| public_error())
    }

    pub fn retryable(&self, id: &str) -> Result<bool, String> {
        store::validate_id(id).map_err(str::to_string)?;
        self.store
            .lock()
            .map_err(|_| public_error())?
            .get(id)
            .map(|operation| operation.status.is_terminal() && operation.can_retry)
            .ok_or_else(|| "update-progress-not-found".to_string())
    }

    pub fn show_window(&self, app: &AppHandle) -> Result<(), String> {
        if self.snapshot()?.is_empty() {
            return Ok(());
        }
        window::show(app, self)
    }

    #[cfg(not(target_os = "linux"))]
    fn saved_position(&self) -> Option<tauri::PhysicalPosition<i32>> {
        self.position.lock().ok().and_then(|position| *position)
    }

    #[cfg(not(target_os = "linux"))]
    fn save_position(&self, position: tauri::PhysicalPosition<i32>) {
        if let Ok(mut saved) = self.position.lock() {
            *saved = Some(position);
        }
    }
}

fn resolve_cancelling_terminal(
    previous: Option<&UpdateOperationSnapshot>,
    operation: &mut UpdateOperationSnapshot,
) {
    if previous.is_some_and(|current| current.status == UpdateOperationStatus::Cancelling)
        && operation.status == UpdateOperationStatus::Failed
    {
        operation.status = UpdateOperationStatus::Cancelled;
        operation.can_retry = false;
        operation.error_key = None;
    }
}

fn should_show_window(
    previous: Option<&UpdateOperationSnapshot>,
    operation: &UpdateOperationSnapshot,
) -> bool {
    !operation.status.is_terminal() && previous.is_none_or(|current| current.status.is_terminal())
}

fn public_error() -> String {
    "update-progress-unavailable".to_string()
}

#[cfg(test)]
mod tests;

pub use types::*;
