use crate::services::forecast::client_http;
use crate::services::forecast::client_local_response;
use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::registry::{find_runtime, ForecastEngineKind};
use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use serde_json::Value;

pub async fn predict(
    base_url: &str,
    auth_token: &str,
    request: &ForecastRequest,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let input = parse_request_input(request)?;
    let payload = build_payload(&input, request)?;

    let client =
        client_http::loopback_client().map_err(|_| "prediction_runtime_failed".to_string())?;
    let http_request = client
        .post(format!("{base_url}/predict"))
        .header("Content-Type", "application/json")
        .header("X-CLGO-Forecast-Token", auth_token)
        .json(&payload);
    let resp = client
        .send(http_request)
        .await
        .map_err(|_| "prediction_runtime_failed".to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("[forecast] requête de prédiction refusée status={status}");
        return Err(if status.is_client_error() {
            "prediction_rejected".to_string()
        } else {
            "prediction_runtime_failed".to_string()
        });
    }

    let body: Value = client_http::read_json(resp)
        .await
        .map_err(|_| "invalid_prediction_output".to_string())?;

    client_local_response::parse_response(&body, request, &input, session_id)
        .map_err(|_| "invalid_prediction_output".to_string())
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let model = request
        .model
        .as_deref()
        .ok_or("Aucun modèle Forecast sélectionné")?;
    let runtime = find_runtime(model).ok_or("Moteur indisponible")?;
    if !crate::services::forecast::registry::has_predict_adapter(runtime) {
        return Err("Moteur indisponible".into());
    }

    let model_config = crate::services::forecast::model_config::effective_values(model)?;
    let quantiles = crate::services::forecast::intervals::configured_levels(
        request.confidence_level,
        &model_config,
    );

    Ok(match runtime.engine_kind {
        ForecastEngineKind::LocalChronosBolt => serde_json::json!({
            "values": input.values.to_vec(),
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "quantiles": quantiles,
            "model_config": model_config,
        }),
        ForecastEngineKind::LocalChronos2 => serde_json::json!({
            "history_rows": input.history_rows,
            "future_rows": input.future_rows,
            "date_column": request.date_column,
            "target_column": request.target_column,
            "series_column": request.series_column,
            "covariate_columns": request.covariate_columns,
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "quantiles": quantiles,
            "model_config": model_config,
        }),
        ForecastEngineKind::LocalTimesFm
        | ForecastEngineKind::LocalToto
        | ForecastEngineKind::LocalMoirai
        | ForecastEngineKind::LocalFlowState
        | ForecastEngineKind::LocalTabPfnTs
        | ForecastEngineKind::LocalTiRex
        | ForecastEngineKind::LocalKairos
        | ForecastEngineKind::LocalSundial => serde_json::json!({
            "values": input.values.to_vec(),
            "history_rows": input.history_rows,
            "future_rows": input.future_rows,
            "date_column": request.date_column,
            "target_column": request.target_column,
            "series_column": request.series_column,
            "covariate_columns": request.covariate_columns,
            "horizon": request.horizon,
            "frequency": request.frequency,
            "model": model,
            "family": runtime.family_id,
            "quantiles": quantiles,
            "model_config": model_config,
        }),
        ForecastEngineKind::CloudApi => {
            return Err("Moteur local invalide".into());
        }
    })
}
