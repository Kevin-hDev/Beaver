use crate::services::forecast::workbench_context::{self, ForecastWorkbenchSnapshot};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn set_forecast_workbench_context(
    app: AppHandle,
    session_id: String,
    analysis_id: Option<String>,
) -> Result<ForecastWorkbenchSnapshot, String> {
    let snapshot = workbench_context::set(session_id, analysis_id).await?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn get_forecast_workbench_context() -> Result<Option<ForecastWorkbenchSnapshot>, String> {
    workbench_context::get().await
}

#[tauri::command]
pub async fn update_forecast_workbench_draft(
    section: String,
) -> Result<crate::services::forecast::workbench_drafts::ForecastWorkbenchDraft, String> {
    workbench_context::update_draft(section).await
}

#[tauri::command]
pub async fn get_forecast_workbench_geometry(
) -> Result<Option<crate::services::forecast::workbench_geometry::ForecastWorkbenchGeometry>, String>
{
    crate::services::forecast::workbench_geometry::get().await
}

#[tauri::command]
pub async fn save_forecast_workbench_geometry(
    geometry: crate::services::forecast::workbench_geometry::ForecastWorkbenchGeometry,
) -> Result<(), String> {
    crate::services::forecast::workbench_geometry::save(geometry).await
}

fn emit_snapshot(app: &AppHandle, snapshot: &ForecastWorkbenchSnapshot) -> Result<(), String> {
    app.emit("forecast-workbench-context-changed", snapshot)
        .map_err(|_| "Impossible d'actualiser Forecast".to_string())
}
