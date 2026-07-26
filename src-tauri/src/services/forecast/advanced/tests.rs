use super::*;
use crate::services::forecast::types::ForecastResult;

fn analysis(values: &[f64], covariates: &[f64], frequency: &str) -> ForecastResult {
    let history: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            serde_json::json!({
                "date": format!("2026-01-{:02}", index + 1),
                "value": value
            })
        })
        .collect();
    let rows: Vec<_> = values
        .iter()
        .zip(covariates)
        .enumerate()
        .map(|(index, (value, covariate))| {
            serde_json::json!({
                "date": format!("2026-01-{:02}", index + 1),
                "value": value,
                "driver": covariate
            })
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "schema_version": 3,
        "revision": 1,
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Test",
        "target_column": "value",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-small",
        "provider": "chronos-bolt",
        "horizon": 1,
        "frequency": frequency,
        "confidence_level": 0.9,
        "input_summary": {"points": values.len(), "start": "2026-01-01", "end": "2026-03-01"},
        "input_data": {
            "columns": ["date", "value", "driver"],
            "date_column": "date",
            "covariate_columns": ["driver"],
            "rows": rows,
            "history": history
        },
        "predictions": [],
        "quantiles": {"q10": [], "q50": [], "q90": []},
        "covariates_used": ["driver"]
    }))
    .unwrap()
}

#[test]
fn computes_real_decomposition_and_residual_anomalies() {
    let mut values: Vec<_> = (0..60)
        .map(|index| 100.0 + index as f64 * 0.2 + (index % 7) as f64)
        .collect();
    values[42] += 80.0;
    let result = analysis(&values, &values, "D");

    let output = analyze(&result);

    assert_eq!(output.decomposition[0].method, "classical_additive");
    assert_eq!(output.decomposition[0].period, 7);
    assert!(output
        .anomalies
        .iter()
        .any(|item| item.date == "2026-01-43"));
}

#[test]
fn computes_chronological_permutation_importance() {
    let covariates: Vec<_> = (0..80).map(|index| (index % 9) as f64).collect();
    let mut value = 20.0;
    let values: Vec<_> = covariates
        .iter()
        .map(|covariate| {
            value += covariate * 1.5;
            value
        })
        .collect();
    let output = analyze(&analysis(&values, &covariates, "D"));

    assert_eq!(output.variable_importance.status, AnalyticsStatus::Ready);
    assert_eq!(output.variable_importance.items[0].name, "driver");
    assert!(output.variable_importance.items[0].score > 0.0);
}

#[test]
fn detects_distribution_drift_between_bounded_windows() {
    let values: Vec<_> = (0..30)
        .map(|index| index as f64 * 0.05)
        .chain((0..30).map(|index| 20.0 + index as f64 * 0.5))
        .collect();
    let output = analyze(&analysis(&values, &values, "D"));

    assert_eq!(output.drift[0].status, AnalyticsStatus::Ready);
    assert!(output.drift[0].detected);
}

#[test]
fn keeps_a_stationary_repeating_distribution_stable() {
    let values: Vec<_> = (0..90).map(|index| (index % 10) as f64).collect();
    let output = analyze(&analysis(&values, &values, "D"));

    assert_eq!(output.drift[0].status, AnalyticsStatus::Ready);
    assert!(!output.drift[0].detected);
}

#[test]
fn keeps_drift_output_serializable_when_variance_appears() {
    let values: Vec<_> = (0..30)
        .map(|_| 10.0)
        .chain((0..30).map(|index| 10.0 + index as f64))
        .collect();
    let output = analyze(&analysis(&values, &values, "D"));

    assert!(output.drift[0].detected);
    assert!(output.drift[0].variance_ratio.is_none());
    assert!(serde_json::to_vec(&output).is_ok());
}

#[test]
fn fails_advanced_sections_closed_when_extreme_values_overflow() {
    let values = vec![f64::MAX; 5];
    let output = analyze(&analysis(&values, &values, "D"));

    assert_eq!(
        output.decomposition[0].status,
        AnalyticsStatus::InsufficientData
    );
    assert!(output.decomposition[0].points.is_empty());
    assert!(serde_json::to_vec(&output).is_ok());
}
