use base64::{engine::general_purpose::STANDARD as B64, Engine};
use tauri::WebviewWindow;

const MAX_BINARY_SIZE: u64 = 50 * 1024 * 1024;
const MAX_SPREADSHEET_SIZE: u64 = 50 * 1024 * 1024;
const DEFAULT_MAX_ROWS: usize = 500;
const HARD_MAX_ROWS: usize = 5000;

#[tauri::command]
pub async fn read_spreadsheet_preview(
    path: String,
    base_dir: Option<String>,
    sheet: Option<String>,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let resolved = super::file_preview::resolve_preview_path(&path, base_dir.as_deref())?;
    read_spreadsheet_path(resolved, sheet, max_rows).await
}

#[tauri::command]
pub async fn read_selected_spreadsheet_preview(
    window: WebviewWindow,
    path: String,
    sheet: Option<String>,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let resolved = super::attachments::require_selected_file(&window, &path, MAX_SPREADSHEET_SIZE)?;
    read_spreadsheet_path(resolved, sheet, max_rows).await
}

pub(crate) async fn read_spreadsheet_path(
    resolved: std::path::PathBuf,
    sheet: Option<String>,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let max = max_rows.unwrap_or(DEFAULT_MAX_ROWS).min(HARD_MAX_ROWS);

    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(|_| "Fichier introuvable".to_string())?;
    if !metadata.is_file() {
        return Err("Chemin invalide".into());
    }
    if metadata.len() > MAX_SPREADSHEET_SIZE {
        return Err("Fichier trop volumineux".into());
    }

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let result = match ext.as_str() {
        "csv" | "tsv" => {
            crate::services::agent_local::tool_spreadsheet_read::read_csv(&resolved, max)
        }
        "xlsx" | "xls" | "ods" | "xlsm" => {
            crate::services::agent_local::tool_spreadsheet_calamine::read_excel(
                &resolved,
                sheet.as_deref(),
                None,
                max,
            )
        }
        _ => Err("Format non supporté".into()),
    };

    result.map(|v| v.to_string())
}

const BINARY_EXTENSIONS: &[&str] = &["docx", "pdf"];

#[tauri::command]
pub async fn read_binary_preview(path: String, base_dir: Option<String>) -> Result<String, String> {
    let resolved = super::file_preview::resolve_preview_path(&path, base_dir.as_deref())?;

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if !BINARY_EXTENSIONS.contains(&ext.as_str()) {
        return Err("Format non supporté pour l'aperçu binaire".into());
    }

    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(|_| "Fichier introuvable".to_string())?;

    if !metadata.is_file() {
        return Err("Chemin invalide".into());
    }
    if metadata.len() > MAX_BINARY_SIZE {
        return Err("Fichier trop volumineux".into());
    }

    let bytes = tokio::fs::read(&resolved)
        .await
        .map_err(|_| "Impossible de lire le fichier".to_string())?;

    Ok(B64.encode(&bytes))
}
