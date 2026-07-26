use crate::services::forecast::{
    catalog, model_config, model_listing, model_manager, selection_policy, sidecar, validation,
};
use serde_json::{Map, Value};
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub fn list_forecast_models() -> Value {
    model_listing::list_models()
}

#[tauri::command]
pub fn get_selected_forecast_model() -> Option<String> {
    selection_policy::get().ok()?.manual_model_id
}

#[tauri::command]
pub fn set_selected_forecast_model(
    app: AppHandle,
    name: String,
) -> Result<selection_policy::ForecastSelectionPolicy, String> {
    let policy = selection_policy::select_manual_model(&name)?;
    app.emit("forecast-selection-policy-changed", &policy)
        .map_err(|_| "Impossible d'actualiser Forecast".to_string())?;
    Ok(policy)
}

#[tauri::command]
pub fn get_forecast_selection_policy() -> Result<selection_policy::ForecastSelectionPolicy, String>
{
    selection_policy::get()
}

#[tauri::command]
pub fn set_forecast_selection_mode(
    app: AppHandle,
    mode: selection_policy::ForecastSelectionMode,
) -> Result<selection_policy::ForecastSelectionPolicy, String> {
    let policy = selection_policy::set_mode(mode)?;
    app.emit("forecast-selection-policy-changed", &policy)
        .map_err(|_| "Impossible d'actualiser Forecast".to_string())?;
    Ok(policy)
}

#[tauri::command]
pub fn set_forecast_auto_cloud_allowed(
    app: AppHandle,
    allowed: bool,
) -> Result<selection_policy::ForecastSelectionPolicy, String> {
    let policy = selection_policy::set_cloud_allowed(allowed)?;
    app.emit("forecast-selection-policy-changed", &policy)
        .map_err(|_| "Impossible d'actualiser Forecast".to_string())?;
    Ok(policy)
}

#[tauri::command]
pub async fn uninstall_forecast_model(app: AppHandle, name: String) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    let chronos = app.state::<sidecar::ChronosSidecar>();
    let _prediction_guard = chronos.lock_prediction().await;
    sidecar::stop_model(chronos.inner(), &name).await;
    model_manager::uninstall(&name).await?;
    let _ = app.emit("forecast-models-changed", ());
    Ok(())
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<Value> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect()
}

#[tauri::command]
pub fn get_forecast_model_config(
    model_id: String,
) -> Result<model_config::ForecastModelConfig, String> {
    model_config::get(&model_id)
}

#[tauri::command]
pub fn set_forecast_model_config(
    model_id: String,
    values: Map<String, Value>,
) -> Result<model_config::ForecastModelConfig, String> {
    model_config::set(&model_id, values)
}
