use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
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
    let chronos = app.state::<sidecar::ChronosSidecar>().inner().clone();
    let operation_sidecar = chronos.clone();
    chronos
        .run_cancellable(move || async move {
            let _prediction_guard = operation_sidecar.lock_prediction().await;
            if !sidecar::stop_model(&operation_sidecar, &name).await {
                return Err("Impossible d'arrêter le service Forecast".to_string());
            }
            model_manager::uninstall(&name).await
        })
        .await?;
    let _ = app.emit("forecast-models-changed", ());
    Ok(())
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<ProviderCatalogEntry> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|provider| {
            ProviderCatalogEntry::new(
                provider.id,
                provider.display_name,
                ProviderCategory::Forecast,
                provider.signup_url,
                Some(provider.base_url),
                None,
            )
        })
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
