use crate::services::forecast::client_quantiles;
use crate::services::forecast::input_data::ParsedInput;
use crate::services::forecast::registry::{find_runtime, ForecastEngineKind};
use crate::services::forecast::target_domain;
use crate::services::forecast::types::{ForecastRequest, ForecastResult, Prediction, Quantiles};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

pub(super) type ParsedPrediction = (Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>);

pub fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
    session_id: Option<&str>,
) -> Result<ForecastResult, String> {
    let model_name = request
        .model
        .as_deref()
        .ok_or("Aucun modèle Forecast sélectionné")?;
    let runtime = find_runtime(model_name).ok_or("Moteur indisponible")?;
    let (mut predictions, mut q10, mut q50, mut q90) = match runtime.engine_kind {
        ForecastEngineKind::LocalChronos2
        | ForecastEngineKind::LocalTimesFm
        | ForecastEngineKind::LocalToto
        | ForecastEngineKind::LocalMoirai
        | ForecastEngineKind::LocalFlowState
        | ForecastEngineKind::LocalTabPfnTs
        | ForecastEngineKind::LocalTiRex
        | ForecastEngineKind::LocalKairos
        | ForecastEngineKind::LocalSundial => {
            parse_structured_or_simple_response(body, request, input)?
        }
        _ => parse_simple_response(body, request, input)?,
    };
    target_domain::apply_non_negative_floor(
        request,
        input,
        &mut predictions,
        &mut q10,
        &mut q50,
        &mut q90,
    );

    let result = ForecastResult {
        schema_version: crate::services::forecast::types::CURRENT_SCHEMA_VERSION,
        revision: crate::services::forecast::types::default_revision(),
        id: Uuid::new_v4().to_string(),
        name: format!("Forecast {}", request.target_column),
        target_column: request.target_column.clone(),
        created_at: Utc::now().to_rfc3339(),
        session_id: session_id.map(|s| s.to_string()),
        model: model_name.to_string(),
        provider: runtime.family_id.to_string(),
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

fn parse_simple_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedPrediction, String> {
    let median = body["median"]
        .as_array()
        .ok_or("Réponse Forecast: champ median manquant")?;

    let predictions: Vec<Prediction> = median
        .iter()
        .enumerate()
        .map(|(i, value)| {
            Ok(Prediction {
                date: input
                    .future_dates
                    .get(i)
                    .cloned()
                    .ok_or("Dates futures invalides")?,
                value: value
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .ok_or("Réponse Forecast: valeur invalide")?,
                series_id: None,
            })
        })
        .collect::<Result<_, String>>()?;

    let q10 = client_quantiles::array_at_level(
        body,
        crate::services::forecast::intervals::lower_level(request.confidence_level),
    );
    let q50: Vec<f64> = predictions.iter().map(|p| p.value).collect();
    let q90 = client_quantiles::array_at_level(
        body,
        crate::services::forecast::intervals::upper_level(request.confidence_level),
    );
    Ok((predictions, q10, q50, q90))
}

fn parse_structured_or_simple_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedPrediction, String> {
    if body["predictions"].is_array() {
        return crate::services::forecast::client_local_structured::parse(body, request, input);
    }
    if request.series_column.is_some() {
        return Err("Réponse Forecast multi-séries invalide".into());
    }
    parse_simple_response(body, request, input)
}
