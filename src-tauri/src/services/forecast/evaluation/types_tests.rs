use super::*;

#[test]
fn backtest_failures_expose_stable_stage_and_retryability() {
    let unsupported = BacktestFailure::from_code("confidence_unsupported");
    assert_eq!(unsupported.stage, "preflight");
    assert!(!unsupported.retryable);

    let runtime = BacktestFailure::from_code("prediction_runtime_failed");
    assert_eq!(runtime.stage, "runtime");
    assert!(runtime.retryable);
}

#[test]
fn unknown_backtest_failures_are_safely_normalized() {
    let failure = BacktestFailure::from_code("raw internal detail");
    assert_eq!(failure.code, "backtest_failed");
    assert_eq!(failure.stage, "runtime");
}
