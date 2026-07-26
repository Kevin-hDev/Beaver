use crate::services::forecast::{catalog, model_details};

#[tauri::command]
pub async fn get_forecast_model_details(
    model_id: String,
) -> Result<model_details::ForecastModelDetails, String> {
    let model = catalog::find_model(&model_id).ok_or("Modèle inconnu".to_string())?;
    let provider = catalog::find_provider(model.provider_id);
    model_details::fetch(model, provider).await
}
