use crate::services::extensions::{
    self, DiscoveryPreferences, ExtensionHostStatus, ExtensionKind, ExtensionView,
};
use tauri::Emitter;

pub(super) mod command_error;

#[tauri::command]
pub async fn list_extensions() -> Result<Vec<ExtensionView>, String> {
    command_error::close(
        command_error::ExtensionCommand::List,
        extensions::list().map(|records| records.into_iter().map(ExtensionView::from).collect()),
    )
}

#[tauri::command]
pub async fn add_local_extension(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExtensionView, String> {
    let result = async {
        let extension = extensions::install_local(&path)?;
        let view = ExtensionView::from(extension.record.clone());
        extensions::add_local(extension.record)?;
        emit_changed(&app);
        Ok(view)
    }
    .await;
    command_error::close(command_error::ExtensionCommand::AddLocal, result)
}

#[tauri::command]
pub async fn install_git_extension(
    app: tauri::AppHandle,
    url: String,
) -> Result<ExtensionView, String> {
    let deadline = extensions::new_stop_deadline();
    let result = extensions::install_git_source(&app, &url, deadline)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::InstallGit, error)
        })
        .map(|record| {
            emit_changed(&app);
            ExtensionView::from(record)
        });
    command_error::close(command_error::ExtensionCommand::InstallGit, result)
}

#[tauri::command]
pub async fn install_npm_extension(
    app: tauri::AppHandle,
    package_spec: String,
) -> Result<ExtensionView, String> {
    let deadline = extensions::new_stop_deadline();
    let result = extensions::install_npm_source(&app, &package_spec, deadline)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::InstallNpm, error)
        })
        .map(|record| {
            emit_changed(&app);
            ExtensionView::from(record)
        });
    command_error::close(command_error::ExtensionCommand::InstallNpm, result)
}

#[tauri::command]
pub async fn update_extension(app: tauri::AppHandle, extension_id: String) -> Result<bool, String> {
    let deadline = extensions::new_stop_deadline();
    let result = extensions::update_managed_extension(&app, &extension_id, deadline)
        .await
        .map_err(|error| extensions::report_operation_error(extensions::Operation::Update, error))
        .map(|record| {
            emit_changed(&app);
            record.sensitive_access_granted
        });
    command_error::close(command_error::ExtensionCommand::Update, result)
}

#[tauri::command]
pub async fn remove_extension(app: tauri::AppHandle, extension_id: String) -> Result<bool, String> {
    let deadline = extensions::new_stop_deadline();
    let result = extensions::uninstall_extension(&extension_id, deadline)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::Uninstall, error)
        });
    emit_changed(&app);
    command_error::close(command_error::ExtensionCommand::Remove, result)
}

#[tauri::command]
pub async fn set_extension_enabled(
    app: tauri::AppHandle,
    extension_id: String,
    enabled: bool,
    trust_confirmed: bool,
) -> Result<bool, String> {
    let deadline = extensions::new_stop_deadline();
    let result = async {
        let reminder = extensions::set_enabled(&extension_id, enabled, trust_confirmed).await?;
        let runtime_result = if enabled {
            extensions::restart(deadline).await
        } else {
            extensions::revoke_extension(&extension_id, deadline)
                .await
                .map(|_| false)
        };
        emit_changed(&app);
        runtime_result.map(|runtime_reminder| reminder || runtime_reminder)
    }
    .await;
    command_error::close(command_error::ExtensionCommand::SetEnabled, result)
}

#[tauri::command]
pub async fn set_extension_show_in_chat(
    app: tauri::AppHandle,
    extension_id: String,
    show_in_chat: bool,
) -> Result<(), String> {
    let result = extensions::set_show_in_chat(&extension_id, show_in_chat).map(|()| {
        emit_changed(&app);
    });
    command_error::close(command_error::ExtensionCommand::SetShowInChat, result)
}

#[tauri::command]
pub async fn reload_extension_host(app: tauri::AppHandle) -> Result<bool, String> {
    let deadline = extensions::new_stop_deadline();
    let result = extensions::restart(deadline).await;
    emit_changed(&app);
    command_error::close(command_error::ExtensionCommand::ReloadHost, result)
}

#[tauri::command]
pub async fn get_extension_host_status() -> Result<ExtensionHostStatus, String> {
    command_error::close(
        command_error::ExtensionCommand::GetHostStatus,
        Ok(extensions::status()),
    )
}

#[tauri::command]
pub async fn get_extension_discovery_preferences() -> Result<DiscoveryPreferences, String> {
    command_error::close(
        command_error::ExtensionCommand::GetDiscoveryPreferences,
        extensions::discovery_preferences(),
    )
}

#[tauri::command]
pub async fn set_extension_discovery_preferences(
    protected_plugin_ids: Vec<String>,
) -> Result<DiscoveryPreferences, String> {
    command_error::close(
        command_error::ExtensionCommand::SetDiscoveryPreferences,
        extensions::set_discovery_preferences(protected_plugin_ids),
    )
}

#[tauri::command]
pub async fn recover_extension_host(app: tauri::AppHandle) -> Result<bool, String> {
    let deadline = extensions::new_stop_deadline();
    let result = async {
        let reminder = extensions::disable_hosted_extensions().await?;
        let runtime_result = extensions::restart(deadline).await;
        emit_changed(&app);
        runtime_result.map(|runtime_reminder| reminder || runtime_reminder)
    }
    .await;
    command_error::close(command_error::ExtensionCommand::RecoverHost, result)
}

#[tauri::command]
pub async fn open_extension_source(extension_id: String) -> Result<(), String> {
    let result = (|| {
        extensions::validate_identifier(&extension_id)?;
        let record = extensions::list()?
            .into_iter()
            .find(|record| record.manifest.id == extension_id)
            .ok_or_else(|| extensions::error_codes::NOT_FOUND.to_string())?;
        if record.kind != ExtensionKind::Local {
            return Err(extensions::error_codes::OPERATION_FAILED.to_string());
        }
        let source = std::path::PathBuf::from(record.source)
            .canonicalize()
            .map_err(|_| extensions::error_codes::NOT_FOUND.to_string())?;
        open::that_detached(source)
            .map_err(|_| extensions::error_codes::OPERATION_FAILED.to_string())
    })();
    command_error::close(command_error::ExtensionCommand::OpenSource, result)
}

pub(super) fn emit_changed(app: &tauri::AppHandle) {
    let _ = app.emit(extensions::CHANGED_EVENT, ());
}

#[cfg(test)]
#[path = "extensions_tests.rs"]
mod tests;
