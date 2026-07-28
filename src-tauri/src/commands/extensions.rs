use crate::services::extensions::{self, ExtensionHostStatus, ExtensionKind, ExtensionView};
use tauri::Emitter;

const CHANGED_EVENT: &str = "fs:extensions-changed";

#[tauri::command]
pub async fn list_extensions() -> Result<Vec<ExtensionView>, String> {
    extensions::list().map(|records| records.into_iter().map(ExtensionView::from).collect())
}

#[tauri::command]
pub async fn add_local_extension(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExtensionView, String> {
    let extension = extensions::install_local(&path)?;
    let view = ExtensionView::from(extension.record.clone());
    extensions::add_local(extension.record)?;
    emit_changed(&app);
    Ok(view)
}

#[tauri::command]
pub async fn install_git_extension(
    app: tauri::AppHandle,
    url: String,
) -> Result<ExtensionView, String> {
    let record = extensions::install_git_source(&app, &url)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::InstallGit, &error)
        })?;
    let view = ExtensionView::from(record);
    emit_changed(&app);
    Ok(view)
}

#[tauri::command]
pub async fn install_npm_extension(
    app: tauri::AppHandle,
    package_spec: String,
) -> Result<ExtensionView, String> {
    let record = extensions::install_npm_source(&app, &package_spec)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::InstallNpm, &error)
        })?;
    let view = ExtensionView::from(record);
    emit_changed(&app);
    Ok(view)
}

#[tauri::command]
pub async fn update_extension(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<ExtensionView, String> {
    let record = extensions::update_managed_extension(&app, &extension_id)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::Update, &error)
        })?;
    let view = ExtensionView::from(record);
    emit_changed(&app);
    Ok(view)
}

#[tauri::command]
pub async fn remove_extension(app: tauri::AppHandle, extension_id: String) -> Result<(), String> {
    let result = extensions::uninstall_extension(&extension_id)
        .await
        .map_err(|error| {
            extensions::report_operation_error(extensions::Operation::Uninstall, &error)
        });
    emit_changed(&app);
    result
}

#[tauri::command]
pub async fn set_extension_enabled(
    app: tauri::AppHandle,
    extension_id: String,
    enabled: bool,
    trust_confirmed: bool,
) -> Result<(), String> {
    extensions::set_enabled(&extension_id, enabled, trust_confirmed)?;
    let result = extensions::restart().await;
    emit_changed(&app);
    result
}

#[tauri::command]
pub async fn set_extension_show_in_chat(
    app: tauri::AppHandle,
    extension_id: String,
    show_in_chat: bool,
) -> Result<(), String> {
    extensions::set_show_in_chat(&extension_id, show_in_chat)?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn reload_extension_host(app: tauri::AppHandle) -> Result<(), String> {
    let result = extensions::restart().await;
    emit_changed(&app);
    result
}

#[tauri::command]
pub async fn get_extension_host_status() -> Result<ExtensionHostStatus, String> {
    Ok(extensions::status())
}

#[tauri::command]
pub async fn recover_extension_host(app: tauri::AppHandle) -> Result<(), String> {
    extensions::disable_hosted_extensions()?;
    let result = extensions::restart().await;
    emit_changed(&app);
    result
}

#[tauri::command]
pub async fn open_extension_source(extension_id: String) -> Result<(), String> {
    extensions::validate_identifier(&extension_id)?;
    let record = extensions::list()?
        .into_iter()
        .find(|record| record.manifest.id == extension_id)
        .ok_or_else(|| "Extension introuvable.".to_string())?;
    if record.kind != ExtensionKind::Local {
        return Err("Aucun dossier local pour ce plugin.".to_string());
    }
    let source = std::path::PathBuf::from(record.source)
        .canonicalize()
        .map_err(|_| "Source d'extension introuvable.".to_string())?;
    open::that_detached(source).map_err(|_| "Impossible d'ouvrir la source.".to_string())
}

fn emit_changed(app: &tauri::AppHandle) {
    let _ = app.emit(CHANGED_EVENT, ());
}
