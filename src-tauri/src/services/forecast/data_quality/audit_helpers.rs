use super::sequences::SequenceFindings;
use super::types::{DataQualityIssue, DataQualitySeverity as Severity};
use crate::services::forecast::limits::{MAX_ISSUE_SAMPLES, MAX_PROFILE_ISSUES};
use crate::services::forecast::types::ForecastRequest;
use chrono::NaiveDateTime;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub(super) fn add_issue(
    issues: &mut Vec<DataQualityIssue>,
    code: &str,
    severity: Severity,
    sample: String,
) {
    if let Some(issue) = issues
        .iter_mut()
        .find(|issue| issue.code == code && issue.severity == severity)
    {
        issue.count = issue.count.saturating_add(1);
        push_sample(issue, sample);
        return;
    }
    if issues.len() >= MAX_PROFILE_ISSUES {
        return;
    }
    let mut issue = DataQualityIssue {
        code: code.to_string(),
        severity,
        count: 1,
        samples: Vec::new(),
    };
    push_sample(&mut issue, sample);
    issues.push(issue);
}

pub(super) fn add_count_issue(
    issues: &mut Vec<DataQualityIssue>,
    code: &str,
    severity: Severity,
    count: usize,
) {
    if count == 0 || issues.len() >= MAX_PROFILE_ISSUES {
        return;
    }
    issues.push(DataQualityIssue {
        code: code.to_string(),
        severity,
        count,
        samples: Vec::new(),
    });
}

pub(super) fn append_sequence_issues(
    issues: &mut Vec<DataQualityIssue>,
    findings: &SequenceFindings,
) {
    add_with_samples(
        issues,
        "duplicate_date",
        Severity::Error,
        findings.duplicates,
        &findings.samples,
    );
    add_with_samples(
        issues,
        "unordered_dates",
        Severity::Error,
        findings.unordered,
        &findings.samples,
    );
    add_count_issue(
        issues,
        "missing_periods",
        Severity::Warning,
        findings.missing,
    );
    add_count_issue(
        issues,
        "frequency_mismatch",
        Severity::Error,
        findings.inconsistent,
    );
    add_count_issue(
        issues,
        "invalid_future_dates",
        Severity::Error,
        findings.invalid_future,
    );
}

pub(super) fn validate_required_columns(
    columns: &BTreeSet<String>,
    request: &ForecastRequest,
    issues: &mut Vec<DataQualityIssue>,
) {
    let required = std::iter::once(&request.target_column)
        .chain(std::iter::once(&request.date_column))
        .chain(request.series_column.iter())
        .chain(request.covariate_columns.iter());
    for column in required {
        if !columns.contains(column) {
            add_issue(
                issues,
                "missing_required_column",
                Severity::Error,
                column.clone(),
            );
        }
    }
    if columns.len() > crate::services::forecast::limits::MAX_INPUT_COLUMNS {
        add_issue(
            issues,
            "too_many_columns",
            Severity::Error,
            columns.len().to_string(),
        );
    }
}

pub(super) fn count_missing_covariates(
    row: &Map<String, Value>,
    request: &ForecastRequest,
) -> usize {
    request
        .covariate_columns
        .iter()
        .filter(|column| match row.get(*column) {
            None | Some(Value::Null) => true,
            Some(Value::String(value)) => value.trim().is_empty(),
            Some(_) => false,
        })
        .count()
}

pub(super) fn date_bounds(bounds: &[(NaiveDateTime, String)]) -> (String, String) {
    let start = bounds
        .iter()
        .min_by_key(|item| item.0)
        .map(|item| item.1.clone());
    let end = bounds
        .iter()
        .max_by_key(|item| item.0)
        .map(|item| item.1.clone());
    (start.unwrap_or_default(), end.unwrap_or_default())
}

fn add_with_samples(
    issues: &mut Vec<DataQualityIssue>,
    code: &str,
    severity: Severity,
    count: usize,
    samples: &[String],
) {
    if count == 0 || issues.len() >= MAX_PROFILE_ISSUES {
        return;
    }
    issues.push(DataQualityIssue {
        code: code.to_string(),
        severity,
        count,
        samples: samples.iter().take(MAX_ISSUE_SAMPLES).cloned().collect(),
    });
}

fn push_sample(issue: &mut DataQualityIssue, sample: String) {
    if sample.is_empty() || issue.samples.len() >= MAX_ISSUE_SAMPLES {
        return;
    }
    let bounded: String = sample.chars().take(80).collect();
    if !issue.samples.contains(&bounded) {
        issue.samples.push(bounded);
    }
}
