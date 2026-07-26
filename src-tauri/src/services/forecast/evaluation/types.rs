use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub analysis_id: String,
    #[serde(default)]
    pub model_ids: Vec<String>,
    #[serde(default)]
    pub max_windows: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastEvaluation {
    pub schema_version: u32,
    pub created_at: String,
    pub horizon: usize,
    pub windows: usize,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub results: Vec<ModelBacktestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBacktestResult {
    pub model_id: String,
    pub kind: BacktestKind,
    pub metrics: Option<BacktestMetrics>,
    pub calibration: Option<IntervalCalibration>,
    #[serde(default)]
    pub folds: Vec<BacktestFoldMetric>,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    pub rank: Option<usize>,
    pub beats_best_baseline: Option<bool>,
    pub warning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<BacktestFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestFailure {
    pub code: String,
    pub stage: String,
    pub retryable: bool,
}

impl BacktestFailure {
    pub fn from_code(code: &str) -> Self {
        let (code, stage, retryable) = match code {
            "model_unavailable"
            | "model_incompatible"
            | "confidence_unsupported"
            | "cloud_not_allowed"
            | "cloud_not_configured"
            | "model_not_installed"
            | "invalid_backtest_data"
            | "invalid_backtest_horizon"
            | "legacy_columns_unavailable" => (code, "preflight", false),
            "resources_unavailable" => (code, "preflight", true),
            "model_start_failed" => (code, "startup", true),
            "prediction_rejected" | "model_request_invalid" => (code, "request", false),
            "invalid_prediction_output" | "incomplete_predictions" | "missing_series" => {
                (code, "output", false)
            }
            "prediction_runtime_failed" | "window_failed" => (code, "runtime", true),
            _ => ("backtest_failed", "runtime", false),
        };
        Self {
            code: code.to_string(),
            stage: stage.to_string(),
            retryable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacktestKind {
    Baseline,
    Model,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub mase: f64,
    pub smape: f64,
    pub mae: f64,
    pub rmse: f64,
    pub bias: f64,
    pub stability: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantile_loss: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalCalibration {
    pub theoretical_coverage: f64,
    pub measured_coverage: f64,
    pub mean_width: f64,
    pub residual_half_width: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestFoldMetric {
    pub index: usize,
    pub train_points: usize,
    pub test_points: usize,
    pub mae: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestIndexSummary {
    pub created_at: String,
    pub horizon: usize,
    pub windows: usize,
    #[serde(default)]
    pub results: Vec<BacktestIndexResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestIndexResult {
    pub model_id: String,
    pub kind: BacktestKind,
    pub metrics: Option<BacktestMetrics>,
    pub calibration: Option<IntervalCalibration>,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    pub beats_best_baseline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<BacktestFailure>,
}

impl ForecastEvaluation {
    pub fn to_index_summary(&self) -> BacktestIndexSummary {
        BacktestIndexSummary {
            created_at: self.created_at.clone(),
            horizon: self.horizon,
            windows: self.windows,
            results: self
                .results
                .iter()
                .map(|result| BacktestIndexResult {
                    model_id: result.model_id.clone(),
                    kind: result.kind,
                    metrics: result.metrics.clone(),
                    calibration: result.calibration.clone(),
                    duration_ms: result.duration_ms,
                    max_memory_mb: result.max_memory_mb,
                    beats_best_baseline: result.beats_best_baseline,
                    failure: result.failure.clone(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
