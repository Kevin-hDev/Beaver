use crate::services::extensions::{self, ExtensionRecoveryState};

#[tauri::command]
pub async fn get_extension_recovery_state() -> Result<ExtensionRecoveryState, String> {
    extensions::extension_recovery::state()
}

#[tauri::command]
pub async fn keep_extension_disabled(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<bool, String> {
    let reminder = extensions::extension_recovery::keep_disabled(&extension_id).await?;
    let runtime_reminder = extensions::restart(extensions::new_stop_deadline()).await?;
    super::extensions::emit_changed(&app);
    Ok(reminder || runtime_reminder)
}

#[tauri::command]
pub async fn retry_extension_load(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<bool, String> {
    let reminder =
        extensions::extension_recovery::retry(&extension_id, extensions::new_stop_deadline())
            .await?;
    super::extensions::emit_changed(&app);
    Ok(reminder)
}

#[tauri::command]
pub async fn discard_extension_loading_marker(app: tauri::AppHandle) -> Result<(), String> {
    extensions::extension_recovery::discard_marker()?;
    super::extensions::emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn restore_extension_recovery_snapshot(app: tauri::AppHandle) -> Result<bool, String> {
    let restored = extensions::restore_recovery_snapshot().await?;
    let reminder = if restored {
        extensions::restart(extensions::new_stop_deadline()).await?
    } else {
        false
    };
    super::extensions::emit_changed(&app);
    Ok(reminder)
}
