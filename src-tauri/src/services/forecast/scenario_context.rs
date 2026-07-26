use super::{
    registry, scenario_context_run,
    types::{ForecastResult, Scenario},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_CONTEXT_ADJUSTMENTS: usize = 12;
const MIN_CONTEXT_ADJUSTMENT: f64 = -95.0;
const MAX_CONTEXT_ADJUSTMENT: f64 = 500.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioCovariateAdjustment {
    pub column: String,
    #[serde(default = "default_adjustment_mode")]
    pub mode: String,
    pub value: f64,
}

fn default_adjustment_mode() -> String {
    "percent".into()
}

pub async fn build(
    analysis: &ForecastResult,
    id: String,
    name: String,
    description: Option<String>,
    adjustments: Vec<ScenarioCovariateAdjustment>,
    target_series_id: Option<String>,
    chronos: Option<&super::sidecar::ChronosSidecar>,
) -> Result<Scenario, String> {
    validate_context_request(analysis, &adjustments, target_series_id.as_deref())?;
    let rows = apply_adjustments(analysis, &adjustments, target_series_id.as_deref())?;
    let forecast = scenario_context_run::rerun(analysis, rows, chronos).await?;

    Ok(Scenario {
        id,
        name,
        description,
        predictions: forecast.predictions,
        quantiles: forecast.quantiles,
        params_modified: serde_json::json!({
            "kind": "context_adjustment",
            "covariate_adjustments": adjustments,
            "target_series_id": target_series_id,
        }),
    })
}

fn validate_context_request(
    analysis: &ForecastResult,
    adjustments: &[ScenarioCovariateAdjustment],
    target_series_id: Option<&str>,
) -> Result<(), String> {
    if adjustments.is_empty() || adjustments.len() > MAX_CONTEXT_ADJUSTMENTS {
        return Err("Modifications de contexte invalides".into());
    }
    let runtime = registry::find_runtime(&analysis.model).ok_or("Moteur indisponible")?;
    if !runtime.capabilities.future_covariates {
        return Err("Scénario contextuel non supporté par ce moteur".into());
    }
    if analysis.input_data.rows.is_empty() {
        return Err("Données source indisponibles".into());
    }
    if analysis.covariates_used.is_empty() {
        return Err("Aucune covariable disponible".into());
    }
    if let Some(series_id) = target_series_id {
        if !analysis
            .input_data
            .series_ids
            .iter()
            .any(|item| item == series_id)
        {
            return Err("Série de scénario invalide".into());
        }
    }
    for adjustment in adjustments {
        validate_adjustment(analysis, adjustment)?;
    }
    Ok(())
}

fn validate_adjustment(
    analysis: &ForecastResult,
    adjustment: &ScenarioCovariateAdjustment,
) -> Result<(), String> {
    if !analysis
        .covariates_used
        .iter()
        .any(|column| column == &adjustment.column)
    {
        return Err("Covariable de scénario invalide".into());
    }
    if adjustment.mode != "percent" && adjustment.mode != "absolute" {
        return Err("Mode de scénario invalide".into());
    }
    if !adjustment.value.is_finite()
        || !(MIN_CONTEXT_ADJUSTMENT..=MAX_CONTEXT_ADJUSTMENT).contains(&adjustment.value)
    {
        return Err("Valeur de scénario invalide".into());
    }
    Ok(())
}

fn apply_adjustments(
    analysis: &ForecastResult,
    adjustments: &[ScenarioCovariateAdjustment],
    target_series_id: Option<&str>,
) -> Result<Vec<Value>, String> {
    let mut changed = 0usize;
    let mut rows = analysis.input_data.rows.clone();
    for row in &mut rows {
        let Some(object) = row.as_object_mut() else {
            return Err("Format de ligne invalide".into());
        };
        if has_target_value(object.get(&analysis.target_column)) {
            continue;
        }
        if !matches_target_series(object, analysis, target_series_id) {
            continue;
        }
        apply_row_adjustments(object, adjustments)?;
        changed += 1;
    }
    if changed == 0 {
        return Err("Aucune ligne future modifiée".into());
    }
    Ok(rows)
}

fn apply_row_adjustments(
    object: &mut serde_json::Map<String, Value>,
    adjustments: &[ScenarioCovariateAdjustment],
) -> Result<(), String> {
    for adjustment in adjustments {
        let value = object
            .get(&adjustment.column)
            .and_then(read_number)
            .ok_or("Covariable future manquante".to_string())?;
        let next_value = if adjustment.mode == "absolute" {
            adjustment.value
        } else {
            value * (1.0 + adjustment.value / 100.0)
        };
        object.insert(adjustment.column.clone(), Value::from(next_value));
    }
    Ok(())
}

fn matches_target_series(
    object: &serde_json::Map<String, Value>,
    analysis: &ForecastResult,
    target_series_id: Option<&str>,
) -> bool {
    let Some(target) = target_series_id else {
        return true;
    };
    let Some(series_column) = analysis.input_data.series_column.as_ref() else {
        return true;
    };
    object
        .get(series_column)
        .and_then(read_series_text)
        .as_deref()
        == Some(target)
}

fn read_series_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn has_target_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) | None => false,
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Number(number)) => number.as_f64().is_some(),
        Some(Value::Bool(_)) => true,
        Some(_) => true,
    }
}

fn read_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => super::numeric_parse::parse_finite_number(text).ok(),
        _ => None,
    }
}
