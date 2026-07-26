use crate::services::agent_local::security;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use super::file_preview_editors::{detect_editors_for_extension, DetectedEditor};

const MAX_PREVIEW_SIZE: u64 = 2 * 1024 * 1024;
const MAX_PATH_LEN: usize = 4096;
const MAX_EXISTENCE_CHECKS: usize = 500;

#[derive(serde::Serialize)]
pub struct PreviewFileExistence {
    path: String,
    exists: bool,
}

#[tauri::command]
pub async fn read_file_preview(path: String, base_dir: Option<String>) -> Result<String, String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(security::sanitize_error)?;
    if !metadata.is_file() {
        return Err("Fichier non supporté".into());
    }
    if metadata.len() > MAX_PREVIEW_SIZE {
        return Err("Fichier trop volumineux pour l'aperçu".into());
    }
    let bytes = tokio::fs::read(&resolved)
        .await
        .map_err(security::sanitize_error)?;
    if bytes.contains(&0) {
        return Err("Fichier non supporté".into());
    }
    String::from_utf8(bytes).map_err(|_| "Fichier non supporté".to_string())
}

#[tauri::command]
pub async fn check_preview_files_exist(
    paths: Vec<String>,
    base_dir: Option<String>,
) -> Result<Vec<PreviewFileExistence>, String> {
    let mut results = Vec::with_capacity(paths.len().min(MAX_EXISTENCE_CHECKS));
    for path in paths.into_iter().take(MAX_EXISTENCE_CHECKS) {
        let exists = preview_file_exists(&path, base_dir.as_deref()).await;
        results.push(PreviewFileExistence { path, exists });
    }
    Ok(results)
}

#[tauri::command]
pub fn detect_editors_for_file(
    path: String,
    base_dir: Option<String>,
) -> Result<Vec<DetectedEditor>, String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    Ok(detect_editors_for_extension(&resolved))
}

#[tauri::command]
pub fn open_preview_file(path: String, base_dir: Option<String>) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    #[cfg(target_os = "macos")]
    return spawn_cmd("open", &[resolved.as_os_str()]);
    #[cfg(target_os = "linux")]
    return spawn_cmd("xdg-open", &[resolved.as_os_str()]);
    #[cfg(target_os = "windows")]
    return spawn_cmd("explorer.exe", &[resolved.as_os_str()]);
}

#[tauri::command]
pub fn open_preview_with_editor(
    path: String,
    base_dir: Option<String>,
    editor_path: String,
) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    validate_editor_path(&editor_path)?;

    #[cfg(target_os = "macos")]
    return spawn_cmd(
        "open",
        &[
            std::ffi::OsStr::new("-a"),
            std::ffi::OsStr::new(&editor_path),
            resolved.as_os_str(),
        ],
    );

    #[cfg(target_os = "windows")]
    return spawn_cmd(&editor_path, &[resolved.as_os_str()]);

    #[cfg(target_os = "linux")]
    return spawn_cmd(
        "gtk-launch",
        &[std::ffi::OsStr::new(&editor_path), resolved.as_os_str()],
    );
}

fn validate_editor_path(editor_path: &str) -> Result<(), String> {
    if editor_path.is_empty() || editor_path.contains('\0') {
        return Err("Éditeur non autorisé".into());
    }

    #[cfg(target_os = "macos")]
    {
        let p = Path::new(editor_path);
        if !p.is_absolute() || p.extension().is_none_or(|e| e != "app") || !p.is_dir() {
            return Err("Éditeur non autorisé".into());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let p = Path::new(editor_path);
        if !p.is_absolute() || p.extension().is_none_or(|e| e != "exe") || !p.is_file() {
            return Err("Éditeur non autorisé".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        if !editor_path.ends_with(".desktop") {
            return Err("Éditeur non autorisé".into());
        }
        if editor_path.contains('/') || editor_path.contains("..") {
            return Err("Éditeur non autorisé".into());
        }
    }

    Ok(())
}

async fn preview_file_exists(path: &str, base_dir: Option<&str>) -> bool {
    let Ok(resolved) = resolve_preview_path(path, base_dir) else {
        return false;
    };
    tokio::fs::metadata(&resolved)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

pub(crate) fn resolve_preview_path(path: &str, base_dir: Option<&str>) -> Result<PathBuf, String> {
    resolve_preview_path_with_roots(path, base_dir, &security::allowed_read_roots())
}

fn resolve_preview_path_with_roots(
    path: &str,
    base_dir: Option<&str>,
    allowed_roots: &[PathBuf],
) -> Result<PathBuf, String> {
    validate_path_text(path)?;
    let raw_path = Path::new(path);
    let joined = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        let base = base_dir.ok_or("Chemin invalide")?;
        validate_path_text(base)?;
        PathBuf::from(base).join(raw_path)
    };
    let working_dir = base_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    security::validate_read_path_in_roots(&joined, &working_dir, allowed_roots)
}

fn validate_path_text(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
        return Err("Chemin invalide".into());
    }
    if Path::new(path)
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("Chemin invalide".into());
    }
    Ok(())
}

fn spawn_cmd(command: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Impossible d'ouvrir l'éditeur".to_string())
}

#[cfg(test)]
#[path = "file_preview_tests.rs"]
mod tests;
