use std::path::PathBuf;

use tauri::WebviewWindow;
use tauri_plugin_fs::FsExt;

use crate::services::attachment_access::{self, RegisteredAttachment};

const HMAC_VAULT_KEY: &str = "attachments.hmac.v1";
const ERROR_CODE: &str = "attachment_access_denied";

#[tauri::command]
pub fn register_attachment_paths(
    window: WebviewWindow,
    paths: Vec<String>,
) -> Result<Vec<RegisteredAttachment>, String> {
    let key = attachment_key()?;
    let scope = window.fs_scope();
    let registered =
        attachment_access::register_paths(&paths, &key, |path| scope.is_allowed(path))?;
    for file in &registered {
        scope
            .allow_file(&file.path)
            .map_err(|_| ERROR_CODE.to_string())?;
    }
    Ok(registered)
}

#[tauri::command]
pub fn restore_attachment_access(
    window: WebviewWindow,
    path: String,
    access_grant: String,
) -> Result<(), String> {
    let key = attachment_key()?;
    let canonical = attachment_access::verify_access_grant(&path, &access_grant, &key)?;
    window
        .fs_scope()
        .allow_file(canonical)
        .map_err(|_| ERROR_CODE.to_string())
}

pub(crate) fn require_selected_file(
    window: &WebviewWindow,
    path: &str,
    max_size: u64,
) -> Result<PathBuf, String> {
    let scope = window.fs_scope();
    attachment_access::selected_file(path, max_size, |candidate| scope.is_allowed(candidate))
}

fn attachment_key() -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    crate::services::api_keys::get_or_create_random_raw(HMAC_VAULT_KEY, 32)
        .map_err(|_| ERROR_CODE.to_string())
}
