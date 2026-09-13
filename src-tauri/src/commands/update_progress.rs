use crate::services::update_progress::{UpdateOperationSnapshot, UpdateProgressRuntime};
use tauri::Emitter;

#[tauri::command]
pub fn list_update_operations(
    runtime: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<Vec<UpdateOperationSnapshot>, String> {
    runtime.snapshot()
}

#[tauri::command]
pub fn dismiss_update_operation(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, UpdateProgressRuntime>,
    id: String,
) -> Result<bool, String> {
    runtime.dismiss(&app, &id)
}

#[tauri::command]
pub fn request_update_operation_retry(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, UpdateProgressRuntime>,
    id: String,
) -> Result<(), String> {
    if !runtime.retryable(&id).unwrap_or(false) {
        return Err("command-not-available".to_string());
    }
    app.emit_to("main", "update-operation-retry-requested", &id)
        .map_err(|_| "update-progress-unavailable".to_string())
}

#[tauri::command]
pub fn resize_update_progress_window(
    window: tauri::WebviewWindow,
    height: u16,
) -> Result<(), String> {
    if !crate::invoke_gate::resize_allowed(window.label(), height) {
        return Err("command-not-available".to_string());
    }
    crate::services::update_progress::window::resize(&window, height)
}

#[tauri::command]
pub fn show_update_progress_window(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<(), String> {
    runtime.show_window(&app)
}
