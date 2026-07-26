use super::types::ForecastRequest;
use std::path::Path;

pub async fn ensure_request_data(
    request: &mut ForecastRequest,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    let Some(raw_path) = request.file_path.as_deref() else {
        return Ok(());
    };

    if request.data.as_deref().is_some_and(is_usable_json_payload) {
        return Ok(());
    }

    request.data = Some(load_file_data(raw_path, base_dir).await?);
    Ok(())
}

async fn load_file_data(raw_path: &str, base_dir: Option<&Path>) -> Result<String, String> {
    let base = base_dir.and_then(|path| path.to_str());
    let resolved = crate::commands::file_preview::resolve_preview_path(raw_path, base)?;

    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(|_| "Fichier introuvable".to_string())?;
    if !metadata.is_file() {
        return Err("Chemin invalide".into());
    }
    if metadata.len() > super::limits::MAX_SPREADSHEET_BYTES {
        return Err("Fichier trop volumineux".into());
    }

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let preview = match ext.as_str() {
        "csv" | "tsv" => crate::services::agent_local::tool_spreadsheet_read::read_csv(
            &resolved,
            super::limits::MAX_INPUT_ROWS,
        ),
        "xlsx" | "xls" | "ods" | "xlsm" => {
            crate::services::agent_local::tool_spreadsheet_calamine::read_excel(
                &resolved,
                None,
                None,
                super::limits::MAX_INPUT_ROWS,
            )
        }
        _ => Err("Format non supporté".into()),
    }?;

    super::spreadsheet_mapping::preview_to_records_json(&preview)
}

fn is_usable_json_payload(data: &str) -> bool {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(trimmed).is_ok_and(|value| value.is_array())
}
