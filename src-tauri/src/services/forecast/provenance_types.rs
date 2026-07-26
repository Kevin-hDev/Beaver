use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::evaluation::types::BacktestIndexSummary;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastSelectionSource {
    #[default]
    Manual,
    Auto,
    ExplicitUserOverride,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastRunStatus {
    #[default]
    Complete,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastDependencyVersions {
    #[serde(default)]
    pub application: String,
    #[serde(default)]
    pub forecast_runtime: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastEffectiveConfig {
    #[serde(default)]
    pub horizon: u32,
    #[serde(default)]
    pub frequency: String,
    #[serde(default)]
    pub confidence_level: f64,
    #[serde(default)]
    pub series_count: usize,
    #[serde(default)]
    pub covariate_count: usize,
    #[serde(default)]
    pub model_parameters: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastProvenance {
    #[serde(default)]
    pub data_fingerprint: String,
    #[serde(default)]
    pub quality_profile_id: Option<String>,
    #[serde(default)]
    pub model_revision: Option<String>,
    #[serde(default)]
    pub dependency_versions: ForecastDependencyVersions,
    #[serde(default)]
    pub effective_config: ForecastEffectiveConfig,
    #[serde(default)]
    pub selection_source: ForecastSelectionSource,
    #[serde(default)]
    pub selection_reason_codes: Vec<String>,
    #[serde(default)]
    pub hardware_class: Option<String>,
    #[serde(default)]
    pub selection_basis: Option<String>,
    #[serde(default)]
    pub backtest: Option<BacktestIndexSummary>,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub status: ForecastRunStatus,
}
