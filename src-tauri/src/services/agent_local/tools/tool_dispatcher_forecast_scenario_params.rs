use crate::services::forecast::scenario_context::ScenarioCovariateAdjustment;
use crate::services::forecast::scenarios::{ScenarioRequest, ScenarioUpdateRequest};
use serde_json::Value;

pub fn create_request(analysis_id: &str, params: &Value) -> Result<ScenarioRequest, String> {
    let params = require_object(params)?;
    let scenario_kind = required_kind(params)?;
    Ok(ScenarioRequest {
        analysis_id: analysis_id.to_string(),
        name: required_text(params, "name")?,
        description: optional_text(params, "description"),
        adjustment_percent: parse_adjustment(params, &scenario_kind)?,
        covariate_adjustments: parse_covariates(params, &scenario_kind)?,
        target_series_id: optional_text(params, "target_series_id"),
        scenario_kind,
    })
}

pub fn update_request(analysis_id: &str, params: &Value) -> Result<ScenarioUpdateRequest, String> {
    let params = require_object(params)?;
    let scenario_kind = required_kind(params)?;
    Ok(ScenarioUpdateRequest {
        analysis_id: analysis_id.to_string(),
        scenario_id: required_text(params, "scenario_id")?,
        name: required_text(params, "name")?,
        description: optional_text(params, "description"),
        adjustment_percent: parse_adjustment(params, &scenario_kind)?,
        covariate_adjustments: parse_covariates(params, &scenario_kind)?,
        target_series_id: optional_text(params, "target_series_id"),
        scenario_kind,
    })
}

fn require_object(params: &Value) -> Result<&serde_json::Map<String, Value>, String> {
    params
        .as_object()
        .ok_or_else(|| "Paramètres de scénario invalides".to_string())
}

fn required_kind(params: &serde_json::Map<String, Value>) -> Result<String, String> {
    let kind = required_text(params, "scenario_kind")?;
    match kind.as_str() {
        "percent_adjustment" | "context_adjustment" => Ok(kind),
        _ => Err("Type de scénario invalide".into()),
    }
}

fn required_text(params: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    let value = params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Paramètres de scénario manquants".to_string())?;
    Ok(value.to_string())
}

fn optional_text(params: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_adjustment(
    params: &serde_json::Map<String, Value>,
    scenario_kind: &str,
) -> Result<f64, String> {
    if scenario_kind != "percent_adjustment" {
        return Ok(0.0);
    }
    let value = params
        .get("adjustment_percent")
        .and_then(Value::as_f64)
        .ok_or_else(|| "Ajustement de scénario manquant".to_string())?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("Ajustement de scénario invalide".into())
    }
}

fn parse_covariates(
    params: &serde_json::Map<String, Value>,
    scenario_kind: &str,
) -> Result<Vec<ScenarioCovariateAdjustment>, String> {
    if scenario_kind != "context_adjustment" {
        return Ok(Vec::new());
    }
    let value = params
        .get("covariate_adjustments")
        .ok_or_else(|| "Modifications de contexte manquantes".to_string())?;
    let adjustments: Vec<ScenarioCovariateAdjustment> =
        serde_json::from_value(value.clone()).map_err(|_| "Modifications de contexte invalides")?;
    if adjustments.is_empty() {
        return Err("Modifications de contexte invalides".into());
    }
    Ok(adjustments)
}
