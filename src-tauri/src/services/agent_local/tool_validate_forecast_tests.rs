use crate::services::agent_local::tool_validate::validate;
use serde_json::json;

#[test]
fn backtest_contract_enforces_model_and_window_bounds() {
    assert!(validate(
        "forecast_backtest",
        &json!({"analysis_id": "analysis", "max_windows": 0})
    )
    .is_err());
    assert!(validate(
        "forecast_backtest",
        &json!({"analysis_id": "analysis", "max_windows": 6})
    )
    .is_err());
    assert!(validate(
        "forecast_backtest",
        &json!({"analysis_id": "analysis", "model_ids": ["a", "b", "c", "d", "e", "f"]})
    )
    .is_err());
}

#[test]
fn comparison_requires_a_bounded_analysis_id() {
    assert!(validate("forecast_compare_models", &json!({})).is_err());
    assert!(validate(
        "forecast_compare_models",
        &json!({"analysis_id": "a".repeat(65)})
    )
    .is_err());
}
