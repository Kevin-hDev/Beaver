use serde::{Deserialize, Serialize};

use crate::services::forecast::types::{Prediction, Quantiles};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsStatus {
    Ready,
    InsufficientData,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedAnalytics {
    pub schema_version: u32,
    pub generated_at: String,
    pub decomposition: Vec<SeriesDecomposition>,
    pub anomalies: Vec<ResidualAnomaly>,
    pub variable_importance: VariableImportanceReport,
    pub drift: Vec<DriftReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesDecomposition {
    pub series_id: Option<String>,
    pub status: AnalyticsStatus,
    pub method: String,
    pub period: usize,
    pub seasonal_strength: Option<f64>,
    pub points: Vec<DecompositionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionPoint {
    pub date: String,
    pub observed: f64,
    pub trend: f64,
    pub seasonal: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualAnomaly {
    pub id: String,
    pub series_id: Option<String>,
    pub date: String,
    pub observed: f64,
    pub expected: f64,
    pub residual: f64,
    pub score: f64,
    pub severity: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableImportanceReport {
    pub status: AnalyticsStatus,
    pub method: String,
    pub reliability: String,
    #[serde(default = "default_variable_scope")]
    pub scope: String,
    pub validation_points: usize,
    pub baseline_mae: Option<f64>,
    pub items: Vec<VariableImportance>,
}

fn default_variable_scope() -> String {
    "all_series".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableImportance {
    pub name: String,
    pub score: f64,
    pub normalized_score: f64,
    pub direction: String,
    pub validation_mae: f64,
    pub permuted_mae: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub series_id: Option<String>,
    pub status: AnalyticsStatus,
    pub method: String,
    pub reference_points: usize,
    pub recent_points: usize,
    pub score: Option<f64>,
    pub mean_shift: Option<f64>,
    pub variance_ratio: Option<f64>,
    pub trend_shift: Option<f64>,
    pub distribution_shift: Option<f64>,
    pub detected: bool,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastEnsemble {
    pub created_at: String,
    pub method: String,
    pub validation_status: String,
    pub members: Vec<EnsembleMember>,
    pub predictions: Vec<Prediction>,
    pub quantiles: Quantiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleMember {
    pub model_id: String,
    pub weight: f64,
    pub backtest_mase: f64,
}
