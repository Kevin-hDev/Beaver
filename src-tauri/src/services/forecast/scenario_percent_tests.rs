//! Tests de build_percent_scenario (PURE) : applique un ajustement en
//! pourcentage aux prédictions et quantiles.

use super::build_percent_scenario;
use crate::services::forecast::types::ForecastResult;

fn result_with_predictions(values: &[f64], quantiles: (f64, f64, f64)) -> ForecastResult {
    serde_json::from_value(serde_json::json!({
        "id": "src",
        "name": "base",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos",
        "provider": "local",
        "horizon": values.len() as u32,
        "frequency": "D",
        "input_summary": {"points": 0, "start": "", "end": ""},
        "predictions": values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                serde_json::json!({"date": format!("2026-01-{:02}", i + 1), "value": v})
            })
            .collect::<Vec<_>>(),
        "quantiles": {
            "q10": [quantiles.0],
            "q50": [quantiles.1],
            "q90": [quantiles.2]
        }
    }))
    .expect("deserialize")
}

#[test]
fn zero_percent_keeps_values_unchanged() {
    let result = result_with_predictions(&[100.0, 200.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "0%".into(), None, 0.0);

    assert_eq!(scenario.predictions[0].value, 100.0);
    assert_eq!(scenario.predictions[1].value, 200.0);
    assert_eq!(scenario.quantiles.q10, vec![80.0]);
}

#[test]
fn positive_percent_increases_values() {
    // +10% → facteur 1.1
    let result = result_with_predictions(&[100.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "+10%".into(), None, 10.0);

    assert!((scenario.predictions[0].value - 110.0).abs() < 1e-9);
    assert!((scenario.quantiles.q50[0] - 110.0).abs() < 1e-9);
    assert!((scenario.quantiles.q10[0] - 88.0).abs() < 1e-9);
}

#[test]
fn negative_percent_decreases_values() {
    // -20% → facteur 0.8
    let result = result_with_predictions(&[100.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "-20%".into(), None, -20.0);

    assert!((scenario.predictions[0].value - 80.0).abs() < 1e-9);
    assert!((scenario.quantiles.q90[0] - 96.0).abs() < 1e-9);
}

#[test]
fn fifty_percent_halves_values() {
    // -50% → facteur 0.5
    let result = result_with_predictions(&[100.0, 200.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "-50%".into(), None, -50.0);

    assert!((scenario.predictions[0].value - 50.0).abs() < 1e-9);
    assert!((scenario.predictions[1].value - 100.0).abs() < 1e-9);
}

#[test]
fn quantiles_all_scaled_consistently() {
    // q10, q50, q90 doivent toutes être multipliées par le même facteur.
    let result = result_with_predictions(&[100.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "+25%".into(), None, 25.0);

    let factor = 1.25;
    assert!((scenario.quantiles.q10[0] - 80.0 * factor).abs() < 1e-9);
    assert!((scenario.quantiles.q50[0] - 100.0 * factor).abs() < 1e-9);
    assert!((scenario.quantiles.q90[0] - 120.0 * factor).abs() < 1e-9);
}

#[test]
fn preserves_id_name_description() {
    let result = result_with_predictions(&[100.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(
        &result,
        "custom-id".into(),
        "Custom Scenario".into(),
        Some("desc".into()),
        5.0,
    );

    assert_eq!(scenario.id, "custom-id");
    assert_eq!(scenario.name, "Custom Scenario");
    assert_eq!(scenario.description.as_deref(), Some("desc"));
}

#[test]
fn params_modified_records_adjustment() {
    let result = result_with_predictions(&[100.0], (80.0, 100.0, 120.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "5%".into(), None, 5.0);

    assert_eq!(scenario.params_modified["kind"], "percent_adjustment");
    assert_eq!(scenario.params_modified["adjustment_percent"], 5.0);
}

#[test]
fn hundred_percent_doubles_values() {
    // +100% → facteur 2.0
    let result = result_with_predictions(&[50.0], (40.0, 50.0, 60.0));
    let scenario = build_percent_scenario(&result, "s1".into(), "+100%".into(), None, 100.0);

    assert!((scenario.predictions[0].value - 100.0).abs() < 1e-9);
    assert!((scenario.quantiles.q90[0] - 120.0).abs() < 1e-9);
}
