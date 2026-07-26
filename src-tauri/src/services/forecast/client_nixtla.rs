use crate::services::forecast::client_http;
use crate::services::forecast::client_nixtla_retry;
use crate::services::forecast::input_data::{parse_request_input, ParsedInput};
use crate::services::forecast::nixtla_multiseries;
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Quantiles};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
use zeroize::Zeroizing;

const API_BASE: &str = "https://api.nixtla.io";
const DEFAULT_MODEL: &str = "timegpt-2-standard";

pub async fn predict(
    api_key: &Zeroizing<String>,
    request: &ForecastRequest,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let input = parse_request_input(request)?;
    let payload = build_payload(&input, request)?;
    let endpoint = endpoint();

    let client = client_http::internet_client()?;
    let resp =
        client_nixtla_retry::post_json_with_retry(&client, &endpoint, api_key, &payload).await?;

    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("[nixtla] erreur {status}");
        return Err("Erreur du service de prédiction".to_string());
    }

    let body: Value = client_http::read_json(resp).await?;

    parse_response(&body, request, &input, session_id)
}

fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    nixtla_multiseries::build_payload(input, request)
}

fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let (predictions, q10, q50, q90) = nixtla_multiseries::parse_response(body, request, input)?;

    let model_name = request.model.as_deref().unwrap_or(DEFAULT_MODEL);

    let result = ForecastResult {
        schema_version: crate::services::forecast::types::CURRENT_SCHEMA_VERSION,
        revision: crate::services::forecast::types::default_revision(),
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: "nixtla".to_string(),
        horizon: request.horizon,
        frequency: request.frequency.clone(),
        confidence_level: request.confidence_level,
        input_summary: input.summary.clone(),
        input_data: input.snapshot.clone(),
        data_profile: Some(input.data_profile.clone()),
        predictions,
        quantiles: Quantiles { q10, q50, q90 },
        covariates_used: request.covariate_columns.clone(),
        metrics: None,
        evaluation: None,
        advanced_analytics: None,
        ensemble: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
        provenance: Default::default(),
    };
    crate::services::forecast::result_validation::validate(&result, request, input)?;
    Ok(result)
}

fn endpoint() -> String {
    format!("{API_BASE}/v2/forecast")
}

pub(super) fn api_model_id(model: &str) -> &str {
    match model {
        "timegpt-2-standard" => "timegpt-2",
        value => value,
    }
}

#[cfg(test)]
#[path = "client_nixtla_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "client_nixtla_response_tests.rs"]
mod response_tests;
