use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct DatedValue {
    pub date: NaiveDateTime,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataQualitySeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityIssue {
    pub code: String,
    pub severity: DataQualitySeverity,
    pub count: usize,
    #[serde(default)]
    pub samples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProfile {
    pub id: String,
    pub created_at: String,
    #[serde(default)]
    pub fingerprint: String,
    pub valid: bool,
    pub target_column: String,
    pub date_column: String,
    pub series_column: Option<String>,
    pub covariate_columns: Vec<String>,
    pub frequency: String,
    pub horizon: u32,
    #[serde(default)]
    pub confidence_level: Option<f64>,
    pub row_count: usize,
    pub history_points: usize,
    pub future_rows: usize,
    pub series_count: usize,
    pub series_ids: Vec<String>,
    pub history_points_by_series: BTreeMap<String, usize>,
    pub start: String,
    pub end: String,
    pub missing_periods: usize,
    pub outlier_count: usize,
    #[serde(default)]
    pub issues: Vec<DataQualityIssue>,
}

impl DataProfile {
    pub fn blocking_error(&self) -> Option<String> {
        self.issues
            .iter()
            .find(|issue| issue.severity == DataQualitySeverity::Error)
            .map(|issue| self.error_message(issue).to_string())
    }

    fn error_message(&self, issue: &DataQualityIssue) -> &'static str {
        if issue.code == "missing_required_column" {
            if issue
                .samples
                .iter()
                .any(|column| column == &self.target_column)
            {
                return "Colonne cible introuvable";
            }
            if issue
                .samples
                .iter()
                .any(|column| column == &self.date_column)
            {
                return "Colonne date introuvable";
            }
            if issue
                .samples
                .iter()
                .any(|column| self.covariate_columns.contains(column))
            {
                return "Covariable introuvable";
            }
        }
        error_message(&issue.code)
    }
}

fn error_message(code: &str) -> &'static str {
    match code {
        "invalid_date" => "Dates invalides",
        "duplicate_date" => "Dates dupliquées",
        "unordered_dates" => "Dates non chronologiques",
        "frequency_mismatch" => "Fréquence incohérente",
        "invalid_numeric_value" => "Colonne cible non numérique",
        "missing_required_column" => "Colonnes requises introuvables",
        "history_after_future" => "Lignes futures invalides",
        "invalid_future_dates" => "Dates futures invalides",
        "invalid_future_rows" => "Lignes futures invalides",
        "insufficient_history" => "Historique insuffisant",
        "too_many_series" => "Trop de séries",
        "prediction_budget_exceeded" => "Volume de prédictions trop important",
        _ => "Données Forecast invalides",
    }
}
