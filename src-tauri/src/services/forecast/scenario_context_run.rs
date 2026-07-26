use super::{
    client_chronos, client_nixtla, model_manager, registry, sidecar,
    types::{ForecastRequest, ForecastResult},
    validation,
};
use serde_json::Value;

pub async fn rerun(
    analysis: &ForecastResult,
    rows: Vec<Value>,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    rerun_with_model(analysis, rows, &analysis.model, chronos).await
}

pub async fn rerun_with_model(
    analysis: &ForecastResult,
    rows: Vec<Value>,
    model_id: &str,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let mut request = build_forecast_request(analysis, rows, model_id)?;
    validation::validate_request(&request)?;
    let profile = super::data_quality::validate_and_bind(&mut request)?;
    let model_id = validation::model_id(&request)?;
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;
    if profile.future_rows > 0
        && !request.covariate_columns.is_empty()
        && !runtime.capabilities.future_covariates
    {
        return Err("Variables futures non supportées par ce moteur".into());
    }
    if !registry::has_predict_adapter(runtime) {
        return Err("Moteur indisponible".into());
    }

    if registry::is_cloud(runtime) {
        let key = crate::services::api_keys::get_key("nixtla")
            .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
        return client_nixtla::predict(&key, &request, None).await;
    }

    if !model_manager::is_ready(model_id) {
        return Err("Modèle non installé".into());
    }
    let spec = super::catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    super::hardware_profile::validate_model_resources(spec)?;
    let chronos = chronos.ok_or("Service de prédiction indisponible")?;
    let _prediction_guard = chronos.lock_prediction().await;
    let endpoint = sidecar::start(chronos, model_id, runtime.family_id)
        .await
        .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
    let prediction = client_chronos::predict(
        &endpoint.base_url,
        endpoint.auth_token.as_str(),
        &request,
        None,
    )
    .await;
    sidecar::schedule_idle_stop(chronos);
    prediction
}

fn build_forecast_request(
    analysis: &ForecastResult,
    rows: Vec<Value>,
    model_id: &str,
) -> Result<ForecastRequest, String> {
    let date_column = analysis
        .input_data
        .date_column
        .clone()
        .or_else(|| infer_date_column(analysis))
        .ok_or("Colonne date introuvable")?;
    let data = serde_json::to_string(&rows).map_err(|_| "Erreur de sérialisation".to_string())?;

    Ok(ForecastRequest {
        data: Some(data),
        file_path: None,
        data_profile_id: None,
        target_column: analysis.target_column.clone(),
        date_column,
        series_column: analysis.input_data.series_column.clone(),
        covariate_columns: analysis.covariates_used.clone(),
        horizon: analysis.horizon,
        frequency: analysis.frequency.clone(),
        model: Some(model_id.to_string()),
        confidence_level: analysis.confidence_level,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    })
}

fn infer_date_column(analysis: &ForecastResult) -> Option<String> {
    ["date", "timestamp", "time"]
        .iter()
        .find(|column| {
            analysis
                .input_data
                .columns
                .iter()
                .any(|item| item == **column)
        })
        .map(|column| (*column).to_string())
}
