mod store;
pub mod types;
pub(crate) mod window;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

const CHANGED_EVENT: &str = "update-operation-changed";

#[derive(Clone, Default)]
pub struct UpdateProgressRuntime {
    store: Arc<Mutex<store::UpdateProgressStore>>,
    position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
}

impl UpdateProgressRuntime {
    #[allow(
        dead_code,
        reason = "called by the four producer projections added next"
    )]
    pub fn upsert(
        &self,
        app: &AppHandle,
        operation: UpdateOperationSnapshot,
    ) -> Result<bool, String> {
        let changed = self
            .store
            .lock()
            .map_err(|_| public_error())?
            .upsert(operation.clone())
            .map_err(str::to_string)?;
        if changed {
            app.emit_to(window::WINDOW_LABEL, CHANGED_EVENT, &operation)
                .map_err(|_| public_error())?;
            if !operation.status.is_terminal() {
                self.show_window(app)?;
            }
        }
        Ok(changed)
    }

    #[allow(
        dead_code,
        reason = "called by the four producer projections added next"
    )]
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
        if removed && self.snapshot()?.is_empty() {
            if let Some(window) = app.get_webview_window(window::WINDOW_LABEL) {
                window.close().map_err(|_| public_error())?;
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

    fn saved_position(&self) -> Option<tauri::PhysicalPosition<i32>> {
        self.position.lock().ok().and_then(|position| *position)
    }

    fn save_position(&self, position: tauri::PhysicalPosition<i32>) {
        if let Ok(mut saved) = self.position.lock() {
            *saved = Some(position);
        }
    }
}

fn public_error() -> String {
    "update-progress-unavailable".to_string()
}

#[cfg(test)]
mod tests;

pub use types::*;
