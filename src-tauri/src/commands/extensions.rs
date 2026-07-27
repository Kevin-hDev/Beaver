use crate::services::extensions::{self, ExtensionHostStatus, ExtensionKind, ExtensionRecord};
use tauri::Emitter;

const CHANGED_EVENT: &str = "fs:extensions-changed";

#[tauri::command]
pub async fn list_extensions() -> Result<Vec<ExtensionRecord>, String> {
    extensions::list()
}

#[tauri::command]
pub async fn add_local_extension(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExtensionRecord, String> {
    let extension = extensions::install_local(&path)?;
    let record = extension.record.clone();
    extensions::add_local(extension.record)?;
    emit_changed(&app);
    Ok(record)
}

#[tauri::command]
pub async fn remove_extension(app: tauri::AppHandle, extension_id: String) -> Result<(), String> {
    extensions::remove(&extension_id)?;
    let result = extensions::start_and_sync().await;
    emit_changed(&app);
    result
}

#[tauri::command]
pub async fn set_extension_enabled(
    app: tauri::AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<(), String> {
    extensions::set_enabled(&extension_id, enabled)?;
    let result = extensions::start_and_sync().await;
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
pub async fn recover_without_user_extensions(app: tauri::AppHandle) -> Result<(), String> {
    extensions::disable_user_extensions()?;
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
