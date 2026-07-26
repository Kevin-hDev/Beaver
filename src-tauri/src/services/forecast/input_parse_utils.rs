use super::input_dates::build_future_dates;
use super::input_series::read_series_id;
use super::numeric_parse::parse_finite_number;
use super::types::{ForecastRequest, Prediction};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

pub fn collect_columns<'a, I>(columns: &mut Vec<String>, keys: I) -> Result<(), String>
where
    I: Iterator<Item = &'a String>,
{
    for key in keys {
        if !columns.iter().any(|existing| existing == key) {
            columns.push(key.clone());
        }
    }
    if columns.len() > super::limits::MAX_INPUT_COLUMNS {
        return Err("Trop de colonnes".into());
    }
    Ok(())
}

pub fn validate_columns(columns: &[String], request: &ForecastRequest) -> Result<(), String> {
    if !columns
        .iter()
        .any(|column| column == &request.target_column)
    {
        return Err("Colonne cible introuvable".into());
    }
    if !columns.iter().any(|column| column == &request.date_column) {
        return Err("Colonne date introuvable".into());
    }
    for covariate in &request.covariate_columns {
        if !columns.iter().any(|column| column == covariate) {
            return Err("Covariable introuvable".into());
        }
    }
    Ok(())
}

pub fn read_target_value(value: Option<&Value>) -> Result<Option<f64>, String> {
    match value {
        Some(Value::Number(number)) => number
            .as_f64()
            .filter(|numeric| numeric.is_finite())
            .map(Some)
            .ok_or("Colonne cible non numérique".into()),
        Some(Value::String(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            parse_finite_number(trimmed)
                .map(Some)
                .map_err(|_| "Colonne cible non numérique".to_string())
        }
        Some(Value::Null) | None => Ok(None),
        _ => Err("Colonne cible non numérique".into()),
    }
}

pub fn validate_multiseries_future_rows(
    future_rows: &[Value],
    history_series_ids: &BTreeSet<String>,
    request: &ForecastRequest,
) -> Result<(), String> {
    if request.series_column.is_none() || future_rows.is_empty() {
        return Ok(());
    }

    let mut counts = HashMap::<String, usize>::new();
    for row in future_rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        let series_id = read_series_id(object, request)?.ok_or("Valeur de série invalide")?;
        *counts.entry(series_id).or_default() += 1;
    }

    if counts.len() != history_series_ids.len() {
        return Err("Lignes futures invalides".into());
    }
    for series_id in history_series_ids {
        if counts.get(series_id).copied() != Some(request.horizon as usize) {
            return Err("Lignes futures invalides".into());
        }
    }
    Ok(())
}

pub fn build_known_or_relative_dates(
    history: &[Prediction],
    future_rows: &[Value],
    request: &ForecastRequest,
) -> Result<Vec<String>, String> {
    if request.series_column.is_some() {
        return Ok(Vec::new());
    }
    if future_rows.is_empty() {
        let last_date = history
            .last()
            .map(|point| point.date.as_str())
            .unwrap_or_default();
        return Ok(build_future_dates(
            last_date,
            &request.frequency,
            request.horizon,
        ));
    }
    if future_rows.len() != request.horizon as usize {
        return Err("Nombre de lignes futures invalide".into());
    }

    future_rows
        .iter()
        .map(|row| {
            row[&request.date_column]
                .as_str()
                .map(|value| value.to_string())
                .ok_or("Colonne date manquante".to_string())
        })
        .collect()
}
