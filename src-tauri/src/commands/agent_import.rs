use crate::services::agent_import::{
    self, AgentSourceSummary, SaveSelectionResult, SourceSelection,
};
use std::time::Duration;
use tauri::Emitter;

const IMPORT_TASK_TIMEOUT: Duration = Duration::from_secs(15);

#[tauri::command]
pub async fn scan_external_agent_sources() -> Result<Vec<AgentSourceSummary>, String> {
    let task = tokio::task::spawn_blocking(|| {
        let home =
            dirs::home_dir().ok_or_else(|| "Dossier utilisateur indisponible".to_string())?;
        Ok(agent_import::public_sources(&home))
    });
    tokio::time::timeout(IMPORT_TASK_TIMEOUT, task)
        .await
        .map_err(|_| "Analyse indisponible".to_string())?
        .map_err(|_| "Analyse indisponible".to_string())?
}

#[tauri::command]
pub async fn save_external_agent_source_selection(
    app: tauri::AppHandle,
    selection: SourceSelection,
    replace_documents: bool,
) -> Result<SaveSelectionResult, String> {
    let task = tokio::task::spawn_blocking(move || {
        let home =
            dirs::home_dir().ok_or_else(|| "Dossier utilisateur indisponible".to_string())?;
        agent_import::save_source_selection(&home, selection, replace_documents)
    });
    let result = task
        .await
        .map_err(|_| "Enregistrement indisponible".to_string())??;
    if result.saved {
        let _ = app.emit("fs:external-agent-sources-changed", ());
        let _ = app.emit("fs:skills-changed", ());
        let _ = app.emit("fs:personality-changed", ());
    }
    Ok(result)
}
