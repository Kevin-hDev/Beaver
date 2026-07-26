use super::input_data::ParsedInput;
use super::types::{ForecastRequest, ForecastResult, Prediction};

pub fn apply_non_negative_floor(
    request: &ForecastRequest,
    input: &ParsedInput,
    predictions: &mut [Prediction],
    q10: &mut [f64],
    q50: &mut [f64],
    q90: &mut [f64],
) {
    if !requires_non_negative_output(request, input) {
        return;
    }
    for prediction in predictions {
        prediction.value = prediction.value.max(0.0);
    }
    clamp_values(q10);
    clamp_values(q50);
    clamp_values(q90);
}

pub fn requires_non_negative_output(request: &ForecastRequest, input: &ParsedInput) -> bool {
    enabled_by_config(request) && is_non_negative_target(request, input)
}

pub(crate) fn apply_saved_non_negative_floor(result: &mut ForecastResult) {
    if !saved_result_requires_non_negative_output(result) {
        return;
    }
    for prediction in &mut result.predictions {
        prediction.value = prediction.value.max(0.0);
    }
    clamp_values(&mut result.quantiles.q10);
    clamp_values(&mut result.quantiles.q50);
    clamp_values(&mut result.quantiles.q90);
}

fn saved_result_requires_non_negative_output(result: &ForecastResult) -> bool {
    let enabled = result
        .provenance
        .effective_config
        .model_parameters
        .get("non_negative_output")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| model_non_negative_enabled(&result.model))
        .unwrap_or(false);
    enabled
        && !result.input_data.history.is_empty()
        && result
            .input_data
            .history
            .iter()
            .all(|point| point.value.is_finite() && point.value >= 0.0)
        && target_name_is_non_negative(&result.target_column)
}

fn enabled_by_config(request: &ForecastRequest) -> bool {
    let Some(model_id) = request.model.as_deref() else {
        return false;
    };
    model_non_negative_enabled(model_id).unwrap_or(false)
}

fn model_non_negative_enabled(model_id: &str) -> Option<bool> {
    super::model_config::effective_values(model_id)
        .ok()
        .and_then(|values| values.get("non_negative_output").and_then(|v| v.as_bool()))
}

fn is_non_negative_target(request: &ForecastRequest, input: &ParsedInput) -> bool {
    input
        .values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
        && target_name_is_non_negative(&request.target_column)
}

fn target_name_is_non_negative(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    if contains_any(
        &normalized,
        &[
            "return",
            "rendement",
            "pct",
            "percent",
            "ratio",
            "margin",
            "marge",
            "delta",
            "variation",
            "temp",
            "temperature",
            "score",
            "indice",
            "index",
        ],
    ) {
        return false;
    }
    contains_any(
        &normalized,
        &[
            "ca", "eur", "revenue", "revenu", "sales", "vente", "amount", "montant", "price",
            "prix", "total", "commande", "order", "count", "qty", "quantite", "quantity", "volume",
        ],
    )
}

fn contains_any(value: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| value.contains(token))
}

fn clamp_values(values: &mut [f64]) {
    for value in values {
        *value = value.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_revenue_bounds_but_not_return_metrics() {
        assert!(target_name_is_non_negative("ca_total_eur"));
        assert!(!target_name_is_non_negative("nasdaq_return_pct"));
    }
}
