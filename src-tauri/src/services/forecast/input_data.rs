use super::data_quality::{audit_request_data, DataProfile};
use super::input_parse_utils::{
    build_known_or_relative_dates, collect_columns, read_target_value, validate_columns,
    validate_multiseries_future_rows,
};
use super::input_series::read_series_id;
use super::types::{ForecastRequest, InputSummary, Prediction};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputSnapshot {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub date_column: Option<String>,
    #[serde(default)]
    pub covariate_columns: Vec<String>,
    #[serde(default)]
    pub rows: Vec<Value>,
    #[serde(default)]
    pub series_column: Option<String>,
    #[serde(default)]
    pub series_ids: Vec<String>,
    #[serde(default)]
    pub history: Vec<Prediction>,
}

#[derive(Debug, Clone)]
pub struct ParsedInput {
    pub values: Vec<f64>,
    pub future_dates: Vec<String>,
    pub summary: InputSummary,
    pub snapshot: InputSnapshot,
    pub history_rows: Vec<Value>,
    pub future_rows: Vec<Value>,
    pub data_profile: DataProfile,
}

pub fn parse_request_input(request: &ForecastRequest) -> Result<ParsedInput, String> {
    let (rows, data_profile) = audit_request_data(request)?;
    if let Some(error) = data_profile.blocking_error() {
        return Err(error);
    }

    let mut columns = Vec::new();
    let mut history = Vec::new();
    let mut values = Vec::new();
    let mut history_rows = Vec::new();
    let mut future_rows = Vec::new();
    let mut future_phase = false;
    let mut future_phase_by_series = HashSet::new();
    let mut history_series_ids = BTreeSet::new();
    let mut series_ids = BTreeSet::new();

    for row in &rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        collect_columns(&mut columns, object.keys())?;
        let series_id = read_series_id(object, request)?;
        let phase_key = series_id.clone().unwrap_or_default();
        if let Some(id) = &series_id {
            series_ids.insert(id.clone());
        }
        let date = object
            .get(&request.date_column)
            .and_then(Value::as_str)
            .ok_or("Colonne date manquante")?;
        match read_target_value(object.get(&request.target_column))? {
            Some(value) => {
                if request.series_column.is_some() {
                    if future_phase_by_series.contains(&phase_key) {
                        return Err("Lignes futures invalides".into());
                    }
                    history_series_ids.insert(phase_key);
                } else if future_phase {
                    return Err("Lignes futures invalides".into());
                }
                history.push(Prediction {
                    date: date.to_string(),
                    value,
                    series_id: series_id.clone(),
                });
                values.push(value);
                history_rows.push(row.clone());
            }
            None => {
                future_phase = true;
                if request.series_column.is_some() {
                    future_phase_by_series.insert(phase_key);
                }
                future_rows.push(row.clone());
            }
        }
    }

    validate_columns(&columns, request)?;
    if history.is_empty() {
        return Err("Aucun point de données historiques".into());
    }

    let future_dates = build_known_or_relative_dates(&history, &future_rows, request)?;
    validate_multiseries_future_rows(&future_rows, &history_series_ids, request)?;
    let start = data_profile.start.clone();
    let end = data_profile.end.clone();

    Ok(ParsedInput {
        values,
        future_dates,
        summary: InputSummary {
            points: history.len(),
            start,
            end,
        },
        snapshot: InputSnapshot {
            columns,
            date_column: Some(request.date_column.clone()),
            covariate_columns: request.covariate_columns.clone(),
            rows,
            series_column: request.series_column.clone(),
            series_ids: series_ids.into_iter().collect(),
            history,
        },
        history_rows,
        future_rows,
        data_profile,
    })
}
