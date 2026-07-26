use super::audit_helpers::*;
use super::profile::{build_profile, SeriesAudit};
use super::types::{DataProfile, DataQualitySeverity as Severity, DatedValue};
use crate::services::forecast::input_dates::parse_input_datetime;
use crate::services::forecast::input_parse_utils::read_target_value;
use crate::services::forecast::input_series::read_series_id;
use crate::services::forecast::limits;
use crate::services::forecast::types::ForecastRequest;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub fn audit_request_data(request: &ForecastRequest) -> Result<(Vec<Value>, DataProfile), String> {
    let data = request.data.as_deref().ok_or("Données JSON requises")?;
    if data.len() > limits::MAX_INLINE_DATA_BYTES {
        return Err("Données trop volumineuses".into());
    }
    let rows: Vec<Value> =
        serde_json::from_str(data).map_err(|_| "Données JSON invalides".to_string())?;
    if rows.is_empty() {
        return Err("Aucun point de données".into());
    }
    if rows.len() > limits::MAX_INPUT_ROWS {
        return Err("Jeu de données trop volumineux".into());
    }
    let profile = audit_rows(&rows, request);
    Ok((rows, profile))
}

pub fn validate_and_bind(request: &mut ForecastRequest) -> Result<DataProfile, String> {
    let (_, profile) = audit_request_data(request)?;
    if let Some(error) = profile.blocking_error() {
        return Err(error);
    }
    request.data_profile_id = Some(profile.id.clone());
    Ok(profile)
}

pub fn audit_rows(rows: &[Value], request: &ForecastRequest) -> DataProfile {
    let mut issues = Vec::new();
    let mut columns = BTreeSet::new();
    let mut series = BTreeMap::<String, SeriesAudit>::new();
    let mut future_started = HashSet::new();
    let mut future_rows = 0usize;
    let mut missing_future_covariates = 0usize;

    for (index, row) in rows.iter().enumerate() {
        let Some(object) = row.as_object() else {
            add_issue(
                &mut issues,
                "invalid_row",
                Severity::Error,
                index.to_string(),
            );
            continue;
        };
        if object.len() > limits::MAX_INPUT_COLUMNS {
            add_issue(
                &mut issues,
                "too_many_columns",
                Severity::Error,
                index.to_string(),
            );
        }
        for (name, value) in object {
            columns.insert(name.clone());
            if value
                .as_str()
                .is_some_and(|text| text.chars().count() > limits::MAX_CELL_CHARS)
            {
                add_issue(&mut issues, "cell_too_long", Severity::Error, name.clone());
            }
        }
        for column in &request.covariate_columns {
            if object.get(column).is_some_and(|value| {
                !matches!(value, Value::Null | Value::Number(_) | Value::Bool(_))
            }) {
                add_issue(
                    &mut issues,
                    "categorical_covariate",
                    Severity::Warning,
                    column.clone(),
                );
            }
        }

        let series_id = match read_series_id(object, request) {
            Ok(Some(id)) => id,
            Ok(None) => "series-1".to_string(),
            Err(_) => {
                add_issue(
                    &mut issues,
                    "invalid_series_id",
                    Severity::Error,
                    index.to_string(),
                );
                continue;
            }
        };
        let raw_date = object.get(&request.date_column).and_then(Value::as_str);
        let Some(date) = raw_date.and_then(parse_input_datetime) else {
            add_issue(
                &mut issues,
                "invalid_date",
                Severity::Error,
                index.to_string(),
            );
            continue;
        };
        let point = DatedValue {
            date,
            raw: raw_date.unwrap_or_default().to_string(),
        };
        match read_target_value(object.get(&request.target_column)) {
            Ok(Some(value)) => {
                if future_started.contains(&series_id) {
                    add_issue(
                        &mut issues,
                        "history_after_future",
                        Severity::Error,
                        point.raw.clone(),
                    );
                }
                let entry = series.entry(series_id).or_default();
                entry.history.push(point);
                entry.values.push(value);
            }
            Ok(None) => {
                future_rows += 1;
                future_started.insert(series_id.clone());
                series.entry(series_id).or_default().future.push(point);
                missing_future_covariates += count_missing_covariates(object, request);
            }
            Err(_) => add_issue(
                &mut issues,
                "invalid_numeric_value",
                Severity::Error,
                index.to_string(),
            ),
        }
    }

    validate_required_columns(&columns, request, &mut issues);
    build_profile(
        rows.len(),
        future_rows,
        missing_future_covariates,
        series,
        request,
        issues,
    )
}
