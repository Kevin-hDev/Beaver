use crate::services::update_notifications::DismissedUpdate;

#[tauri::command]
pub fn list_dismissed_update_notifications() -> Result<Vec<DismissedUpdate>, String> {
    crate::services::update_notifications::read()
}

#[tauri::command]
pub fn dismiss_update_notification(
    update: DismissedUpdate,
) -> Result<Vec<DismissedUpdate>, String> {
    crate::services::update_notifications::dismiss(update)
}
