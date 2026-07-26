use super::audit_helpers::*;
use super::sequences;
use super::stats;
use super::types::{DataProfile, DataQualityIssue, DataQualitySeverity as Severity, DatedValue};
use crate::services::forecast::limits;
use crate::services::forecast::types::ForecastRequest;
use chrono::Utc;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Default)]
pub(super) struct SeriesAudit {
    pub history: Vec<DatedValue>,
    pub future: Vec<DatedValue>,
    pub values: Vec<f64>,
}

pub(super) fn build_profile(
    row_count: usize,
    future_rows: usize,
    missing_future_covariates: usize,
    series: BTreeMap<String, SeriesAudit>,
    request: &ForecastRequest,
    mut issues: Vec<DataQualityIssue>,
) -> DataProfile {
    let mut missing_periods = 0usize;
    let mut outliers = 0usize;
    let mut history_points_by_series = BTreeMap::new();
    let mut bounds = Vec::new();
    for (id, data) in &series {
        let findings = sequences::inspect(&data.history, &data.future, &request.frequency);
        append_sequence_issues(&mut issues, &findings);
        missing_periods = missing_periods.saturating_add(findings.missing);
        outliers += stats::outlier_count(&data.values);
        if stats::has_regime_shift(&data.values) {
            add_issue(
                &mut issues,
                "possible_regime_shift",
                Severity::Warning,
                id.clone(),
            );
        }
        if data.history.len() < 2 {
            add_issue(
                &mut issues,
                "insufficient_history",
                Severity::Error,
                id.clone(),
            );
        } else if data.history.len() < (request.horizon as usize).saturating_mul(2) {
            add_issue(&mut issues, "short_history", Severity::Warning, id.clone());
        }
        if future_rows > 0 && data.future.len() != request.horizon as usize {
            add_issue(
                &mut issues,
                "invalid_future_rows",
                Severity::Error,
                id.clone(),
            );
        }
        history_points_by_series.insert(id.clone(), data.history.len());
        bounds.extend(
            data.history
                .iter()
                .map(|point| (point.date, point.raw.clone())),
        );
    }
    validate_size_limits(&series, request, &mut issues);
    if missing_future_covariates > 0 {
        add_count_issue(
            &mut issues,
            "missing_future_covariates",
            Severity::Warning,
            missing_future_covariates,
        );
    }
    if outliers > 0 {
        add_count_issue(
            &mut issues,
            "possible_outliers",
            Severity::Warning,
            outliers,
        );
    }
    let (start, end) = date_bounds(&bounds);
    let valid = !issues.iter().any(|issue| issue.severity == Severity::Error);
    DataProfile {
        id: profile_id(request),
        created_at: Utc::now().to_rfc3339(),
        fingerprint: crate::services::forecast::data_fingerprint::for_request(request),
        valid,
        target_column: request.target_column.clone(),
        date_column: request.date_column.clone(),
        series_column: request.series_column.clone(),
        covariate_columns: request.covariate_columns.clone(),
        frequency: request.frequency.clone(),
        horizon: request.horizon,
        confidence_level: Some(request.confidence_level),
        row_count,
        history_points: history_points_by_series.values().sum(),
        future_rows,
        series_count: series.len().max(1),
        series_ids: request
            .series_column
            .as_ref()
            .map_or_else(Vec::new, |_| series.keys().cloned().collect()),
        history_points_by_series,
        start,
        end,
        missing_periods,
        outlier_count: outliers,
        issues,
    }
}

fn validate_size_limits(
    series: &BTreeMap<String, SeriesAudit>,
    request: &ForecastRequest,
    issues: &mut Vec<DataQualityIssue>,
) {
    if series.is_empty() {
        add_issue(
            issues,
            "insufficient_history",
            Severity::Error,
            String::new(),
        );
    }
    if series.len() > limits::MAX_SERIES {
        add_issue(
            issues,
            "too_many_series",
            Severity::Error,
            series.len().to_string(),
        );
    }
    if limits::validate_prediction_budget(series.len().max(1), request.horizon).is_err() {
        add_issue(
            issues,
            "prediction_budget_exceeded",
            Severity::Error,
            String::new(),
        );
    }
}

fn profile_id(request: &ForecastRequest) -> String {
    request
        .data_profile_id
        .as_deref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::new_v4)
        .to_string()
}
