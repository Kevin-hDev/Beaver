use super::types::{ForecastResult, Quantiles, Scenario};

pub fn build_percent_scenario(
    analysis: &ForecastResult,
    id: String,
    name: String,
    description: Option<String>,
    adjustment_percent: f64,
) -> Scenario {
    let factor = 1.0 + adjustment_percent / 100.0;
    Scenario {
        id,
        name,
        description,
        predictions: analysis
            .predictions
            .iter()
            .map(|point| {
                let mut adjusted = point.clone();
                adjusted.value *= factor;
                adjusted
            })
            .collect(),
        quantiles: scale_quantiles(&analysis.quantiles, factor),
        params_modified: serde_json::json!({
            "kind": "percent_adjustment",
            "adjustment_percent": adjustment_percent,
        }),
    }
}

fn scale_quantiles(quantiles: &Quantiles, factor: f64) -> Quantiles {
    Quantiles {
        q10: scale_values(&quantiles.q10, factor),
        q50: scale_values(&quantiles.q50, factor),
        q90: scale_values(&quantiles.q90, factor),
    }
}

fn scale_values(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter().map(|value| value * factor).collect()
}

#[cfg(test)]
#[path = "scenario_percent_tests.rs"]
mod tests;
