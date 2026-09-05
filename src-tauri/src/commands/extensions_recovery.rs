use crate::services::extensions::{self, ExtensionRecoveryState};

#[tauri::command]
pub async fn get_extension_recovery_state() -> Result<ExtensionRecoveryState, String> {
    super::extensions::command_error::close(
        super::extensions::command_error::ExtensionCommand::GetRecoveryState,
        extensions::extension_recovery::state(),
    )
}

#[tauri::command]
pub async fn keep_extension_disabled(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<bool, String> {
    let result = async {
        let reminder = extensions::extension_recovery::keep_disabled(&extension_id).await?;
        let runtime_reminder = extensions::restart(extensions::new_stop_deadline()).await?;
        super::extensions::emit_changed(&app);
        Ok(reminder || runtime_reminder)
    }
    .await;
    super::extensions::command_error::close(
        super::extensions::command_error::ExtensionCommand::KeepDisabled,
        result,
    )
}

#[tauri::command]
pub async fn retry_extension_load(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<bool, String> {
    let result = extensions::extension_recovery::retry(&extension_id)
        .await
        .inspect(|_| {
            super::extensions::emit_changed(&app);
        });
    super::extensions::command_error::close(
        super::extensions::command_error::ExtensionCommand::RetryLoad,
        result,
    )
}

#[tauri::command]
pub async fn discard_extension_loading_marker(app: tauri::AppHandle) -> Result<(), String> {
    let result = extensions::extension_recovery::discard_marker().map(|()| {
        super::extensions::emit_changed(&app);
    });
    super::extensions::command_error::close(
        super::extensions::command_error::ExtensionCommand::DiscardLoadingMarker,
        result,
    )
}

#[tauri::command]
pub async fn restore_extension_recovery_snapshot(app: tauri::AppHandle) -> Result<bool, String> {
    let result = async {
        let restored = extensions::restore_recovery_snapshot().await?;
        let reminder = if restored {
            extensions::restart(extensions::new_stop_deadline()).await?
        } else {
            false
        };
        super::extensions::emit_changed(&app);
        Ok(reminder)
    }
    .await;
    super::extensions::command_error::close(
        super::extensions::command_error::ExtensionCommand::RestoreRecoverySnapshot,
        result,
    )
}
