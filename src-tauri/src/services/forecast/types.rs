pub use super::annotation_types::{Annotation, AnnotationSource};
use super::evaluation::types::BacktestIndexSummary;
use super::evaluation::types::ForecastEvaluation;
use super::input_data::InputSnapshot;
use super::provenance_types::{ForecastProvenance, ForecastSelectionSource};
use serde::{Deserialize, Serialize};

pub const MAX_ANNOTATIONS: usize = 200;
pub const MAX_SCENARIOS: usize = 50;
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRequest {
    pub data: Option<String>,
    pub file_path: Option<String>,
    #[serde(default)]
    pub data_profile_id: Option<String>,
    pub target_column: String,
    pub date_column: String,
    pub series_column: Option<String>,
    #[serde(default)]
    pub covariate_columns: Vec<String>,
    pub horizon: u32,
    pub frequency: String,
    pub model: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_level: f64,
    #[serde(default)]
    pub selection_id: Option<String>,
    #[serde(default)]
    pub selection_source: Option<ForecastSelectionSource>,
    #[serde(default)]
    pub selection_reason_codes: Vec<String>,
}

pub fn default_confidence() -> f64 {
    0.9
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_revision")]
    pub revision: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub target_column: String,
    pub created_at: String,
    pub session_id: Option<String>,
    pub model: String,
    pub provider: String,
    pub horizon: u32,
    pub frequency: String,
    #[serde(default = "default_confidence")]
    pub confidence_level: f64,
    pub input_summary: InputSummary,
    #[serde(default)]
    pub input_data: InputSnapshot,
    #[serde(default)]
    pub data_profile: Option<super::data_quality::DataProfile>,
    pub predictions: Vec<Prediction>,
    pub quantiles: Quantiles,
    #[serde(default)]
    pub covariates_used: Vec<String>,
    pub metrics: Option<ForecastMetrics>,
    #[serde(default)]
    pub evaluation: Option<ForecastEvaluation>,
    #[serde(default)]
    pub advanced_analytics: Option<super::advanced::AdvancedAnalytics>,
    #[serde(default)]
    pub ensemble: Option<super::advanced::ForecastEnsemble>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
    #[serde(default)]
    pub provenance: ForecastProvenance,
}

pub fn default_schema_version() -> u32 {
    1
}

pub fn default_revision() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSummary {
    pub points: usize,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub date: String,
    pub value: f64,
    #[serde(default)]
    pub series_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantiles {
    #[serde(default)]
    pub q10: Vec<f64>,
    #[serde(default)]
    pub q50: Vec<f64>,
    #[serde(default)]
    pub q90: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub mape: Option<f64>,
    pub mae: Option<f64>,
    pub crps: Option<f64>,
    pub bias: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub predictions: Vec<Prediction>,
    pub quantiles: Quantiles,
    pub params_modified: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastAnalysisMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub model: String,
    pub provider: String,
    pub horizon: u32,
    pub frequency: String,
    #[serde(default)]
    pub confidence_level: Option<f64>,
    pub points: usize,
    pub mape: Option<f64>,
    pub session_id: Option<String>,
    #[serde(default)]
    pub scenarios_count: usize,
    #[serde(default)]
    pub data_profile_id: Option<String>,
    #[serde(default)]
    pub data_fingerprint: String,
    #[serde(default)]
    pub backtest: Option<BacktestIndexSummary>,
}

impl ForecastResult {
    pub fn to_meta(&self) -> ForecastAnalysisMeta {
        ForecastAnalysisMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at.clone(),
            model: self.model.clone(),
            provider: self.provider.clone(),
            horizon: self.horizon,
            frequency: self.frequency.clone(),
            confidence_level: Some(self.confidence_level),
            points: self.input_summary.points,
            mape: self.metrics.as_ref().and_then(|m| m.mape),
            session_id: self.session_id.clone(),
            scenarios_count: self.scenarios.len(),
            data_profile_id: self.data_profile.as_ref().map(|profile| profile.id.clone()),
            data_fingerprint: self
                .data_profile
                .as_ref()
                .map(|profile| profile.fingerprint.clone())
                .unwrap_or_default(),
            backtest: self
                .evaluation
                .as_ref()
                .map(ForecastEvaluation::to_index_summary),
        }
    }
}
