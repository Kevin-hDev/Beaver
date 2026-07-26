use super::*;

fn analysis(model: &str, value: f64) -> ForecastResult {
    serde_json::from_value(serde_json::json!({
        "schema_version": 3,
        "revision": 1,
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Ensemble",
        "target_column": "value",
        "created_at": "2026-07-23T00:00:00Z",
        "model": model,
        "provider": "test",
        "horizon": 1,
        "frequency": "D",
        "confidence_level": 0.9,
        "input_summary": {"points": 30, "start": "2026-01-01", "end": "2026-01-30"},
        "input_data": {"history": []},
        "predictions": [{"date": "2026-01-31", "value": value}],
        "quantiles": {"q10": [value - 2.0], "q50": [value], "q90": [value + 2.0]},
        "evaluation": {
            "schema_version": 1,
            "created_at": "2026-07-23T00:00:00Z",
            "horizon": 1,
            "windows": 3,
            "results": [
                model_result("model-a", 0.5, 1),
                model_result("model-b", 1.0, 2)
            ]
        }
    }))
    .unwrap()
}

fn model_result(model: &str, mase: f64, rank: usize) -> serde_json::Value {
    serde_json::json!({
        "model_id": model,
        "kind": "model",
        "metrics": {
            "mase": mase, "smape": 5.0, "mae": 1.0, "rmse": 1.0,
            "bias": 0.0, "stability": 0.1, "quantile_loss": 0.2
        },
        "calibration": null,
        "folds": [],
        "duration_ms": 10,
        "rank": rank,
        "beats_best_baseline": true,
        "warning": null
    })
}

#[test]
fn selects_only_successful_backtested_models_and_normalizes_weights() {
    let members = select_members(&analysis("model-a", 10.0), &[]).unwrap();
    assert_eq!(members.len(), 2);
    assert!(members[0].weight > members[1].weight);
    assert!((members.iter().map(|member| member.weight).sum::<f64>() - 1.0).abs() < 1e-9);
}

#[test]
fn combines_aligned_predictions_with_inverse_mase_weights() {
    let first = analysis("model-a", 10.0);
    let second = analysis("model-b", 20.0);
    let members = select_members(&first, &[]).unwrap();

    let ensemble = weighted(&[first, second], members).unwrap();

    assert!((ensemble.predictions[0].value - 13.333_333).abs() < 1e-5);
    assert_eq!(
        ensemble.validation_status,
        "members_backtested_ensemble_not_backtested"
    );
}

#[test]
fn rejects_duplicate_or_single_model_requests() {
    let output = analysis("model-a", 10.0);
    assert!(select_members(&output, &["model-a".into()]).is_err());
    assert!(select_members(&output, &["model-a".into(), "model-a".into()]).is_err());
}

#[test]
fn rejects_incomplete_or_misaligned_member_outputs_before_combining() {
    let first = analysis("model-a", 10.0);
    let mut second = analysis("model-b", 20.0);
    second.quantiles.q90.clear();
    assert!(validate_alignment(&[first.clone(), second]).is_err());

    let mut shifted = analysis("model-b", 20.0);
    shifted.predictions[0].date = "2026-02-01".into();
    assert!(validate_alignment(&[first, shifted]).is_err());
}

#[test]
fn rejects_non_finite_weighted_outputs() {
    let first = analysis("model-a", f64::MAX);
    let second = analysis("model-b", f64::MAX);
    let members = vec![
        EnsembleMember {
            model_id: "model-a".into(),
            weight: 1.0,
            backtest_mase: 0.5,
        },
        EnsembleMember {
            model_id: "model-b".into(),
            weight: 1.0,
            backtest_mase: 1.0,
        },
    ];

    assert!(weighted(&[first, second], members).is_err());
}
