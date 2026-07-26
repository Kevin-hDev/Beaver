use crate::services::forecast::dev_updates::{self, ForecastDevUpdate};

#[tauri::command]
pub async fn check_forecast_dev_updates() -> Result<Vec<ForecastDevUpdate>, String> {
    dev_updates::check().await
}
