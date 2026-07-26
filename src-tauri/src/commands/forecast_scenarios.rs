use crate::services::forecast::types::ForecastResult;
use crate::services::forecast::{scenarios, sidecar};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn create_forecast_scenario(
    app: AppHandle,
    request: scenarios::ScenarioRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let analysis = scenarios::create(request, Some(chronos.inner())).await?;
    emit_updated(&app, &analysis);
    Ok(analysis)
}

#[tauri::command]
pub async fn update_forecast_scenario(
    app: AppHandle,
    request: scenarios::ScenarioUpdateRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let analysis = scenarios::update(request, Some(chronos.inner())).await?;
    emit_updated(&app, &analysis);
    Ok(analysis)
}

#[tauri::command]
pub async fn delete_forecast_scenario(
    app: AppHandle,
    analysis_id: String,
    scenario_id: String,
) -> Result<ForecastResult, String> {
    let analysis = scenarios::delete(&analysis_id, &scenario_id).await?;
    emit_updated(&app, &analysis);
    Ok(analysis)
}

fn emit_updated(app: &AppHandle, analysis: &ForecastResult) {
    crate::services::forecast::events::emit_updated(app, analysis);
}
