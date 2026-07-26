use super::client_nixtla_options;
use super::input_data::ParsedInput;
use super::types::{ForecastRequest, Prediction};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[path = "nixtla_exogenous.rs"]
mod exogenous;

type ParsedNixtla = (Vec<Prediction>, Vec<f64>, Vec<f64>, Vec<f64>);

pub fn build_payload(input: &ParsedInput, request: &ForecastRequest) -> Result<Value, String> {
    let grouped_history = group_rows(&input.history_rows, request.series_column.as_deref())?;
    let grouped_future = group_rows(&input.future_rows, request.series_column.as_deref())?;
    if grouped_history.is_empty() {
        return Err("Données de prédiction invalides".into());
    }
    validate_future_rows(&grouped_history, &grouped_future, input, request)?;

    let mut y = Vec::<f64>::new();
    let mut sizes = Vec::<usize>::new();
    let mut x = Vec::<Vec<Value>>::new();
    let mut x_future = Vec::<Vec<Value>>::new();
    for (series_id, history_rows) in &grouped_history {
        sizes.push(history_rows.len());
        for row in history_rows {
            let value =
                super::input_parse_utils::read_target_value(row.get(&request.target_column))?
                    .ok_or("Colonne cible non numérique")?;
            y.push(value);
            if !request.covariate_columns.is_empty() {
                x.push(exogenous::read_row(row, &request.covariate_columns)?);
            }
        }
        if let Some(future_rows) = grouped_future.get(series_id) {
            for row in future_rows {
                if !request.covariate_columns.is_empty() {
                    x_future.push(exogenous::read_row(row, &request.covariate_columns)?);
                }
            }
        }
    }
    let mut series = json!({ "y": y, "sizes": sizes });
    if !request.covariate_columns.is_empty() {
        let x_columns = exogenous::transpose(x, request.covariate_columns.len())?;
        let future_columns = exogenous::transpose(x_future, request.covariate_columns.len())?;
        let categorical = exogenous::categorical_indices(&x_columns, &future_columns)?;
        series["X"] = Value::Array(x_columns);
        series["X_future"] = Value::Array(future_columns);
        if !categorical.is_empty() {
            series["categorical_exog"] = json!(categorical);
        }
    }
    let ui_model = request.model.as_deref().unwrap_or("timegpt-2-standard");
    let config = client_nixtla_options::effective_config(ui_model);
    let level = client_nixtla_options::effective_level(request.confidence_level);
    let mut payload = json!({
        "series": series,
        "freq": request.frequency,
        "h": request.horizon,
        "model": super::client_nixtla::api_model_id(ui_model),
        "level": [level],
        "clean_ex_first": true,
    });
    if ui_model == "timegpt-2.1" && groups_are_aligned(&grouped_history, &grouped_future, request) {
        payload["multivariate"] = Value::Bool(true);
    }
    client_nixtla_options::apply(&mut payload, &config);
    Ok(payload)
}
pub fn parse_response(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedNixtla, String> {
    let mean = body["mean"]
        .as_array()
        .ok_or("Réponse de prédiction invalide")?;
    let level = client_nixtla_options::effective_level(request.confidence_level);
    let lower = client_nixtla_options::interval_array(body, &format!("lo-{level}"))
        .map_err(|_| "Intervalles de prédiction invalides")?;
    let upper = client_nixtla_options::interval_array(body, &format!("hi-{level}"))
        .map_err(|_| "Intervalles de prédiction invalides")?;
    let dates = super::input_future::expected_dates_by_series(request, input)?;
    let horizon = request.horizon as usize;
    let expected_count = dates.len().saturating_mul(horizon);
    if mean.len() != expected_count
        || lower.len() != expected_count
        || upper.len() != expected_count
    {
        return Err("Réponse de prédiction incomplète".into());
    }
    let mut predictions = Vec::with_capacity(mean.len());
    let mut q10 = Vec::with_capacity(mean.len());
    let mut q50 = Vec::with_capacity(mean.len());
    let mut q90 = Vec::with_capacity(mean.len());
    let mut offset = 0usize;
    for (series_id, series_dates) in dates {
        for index in 0..horizon {
            let value = mean
                .get(offset)
                .and_then(Value::as_f64)
                .filter(|number| number.is_finite())
                .ok_or("Réponse de prédiction invalide")?;
            predictions.push(Prediction {
                date: series_dates
                    .get(index)
                    .cloned()
                    .ok_or("Dates futures invalides")?,
                value,
                series_id: request.series_column.as_ref().map(|_| series_id.clone()),
            });
            q10.push(interval_at(&lower, offset)?);
            q50.push(value);
            q90.push(interval_at(&upper, offset)?);
            offset += 1;
        }
    }
    Ok((predictions, q10, q50, q90))
}

fn interval_at(values: &[f64], offset: usize) -> Result<f64, String> {
    values
        .get(offset)
        .copied()
        .ok_or("Intervalle de prédiction incomplet".to_string())
}

fn group_rows(
    rows: &[Value],
    series_column: Option<&str>,
) -> Result<BTreeMap<String, Vec<Map<String, Value>>>, String> {
    let mut grouped = BTreeMap::<String, Vec<Map<String, Value>>>::new();
    for row in rows {
        let object = row.as_object().ok_or("Format de ligne invalide")?;
        let series_id = match series_column {
            Some(column) => {
                let value = object.get(column).ok_or("Valeur de série invalide")?;
                super::input_series::normalize_series_value(value)?
                    .ok_or("Valeur de série invalide")?
            }
            None => "series-1".to_string(),
        };
        grouped.entry(series_id).or_default().push(object.clone());
    }
    Ok(grouped)
}

fn validate_future_rows(
    history: &BTreeMap<String, Vec<Map<String, Value>>>,
    future: &BTreeMap<String, Vec<Map<String, Value>>>,
    input: &ParsedInput,
    request: &ForecastRequest,
) -> Result<(), String> {
    if future.is_empty() && !request.covariate_columns.is_empty() {
        return Err("Données de contexte futur invalides".into());
    }
    if future.is_empty() {
        return Ok(());
    }
    let horizon = request.horizon as usize;
    if future.len() != history.len()
        || history
            .keys()
            .any(|id| future.get(id).is_none_or(|rows| rows.len() != horizon))
        || input.future_rows.len() != history.len().saturating_mul(horizon)
    {
        return Err("Données de contexte futur invalides".into());
    }
    Ok(())
}

fn groups_are_aligned(
    history: &BTreeMap<String, Vec<Map<String, Value>>>,
    future: &BTreeMap<String, Vec<Map<String, Value>>>,
    request: &ForecastRequest,
) -> bool {
    history.len() > 1
        && aligned_dates(history, &request.date_column)
        && (future.is_empty() || aligned_dates(future, &request.date_column))
}

fn aligned_dates(groups: &BTreeMap<String, Vec<Map<String, Value>>>, date_column: &str) -> bool {
    let mut sequences = groups.values().map(|rows| {
        rows.iter()
            .map(|row| row.get(date_column).and_then(Value::as_str))
            .collect::<Option<Vec<_>>>()
    });
    let Some(Some(reference)) = sequences.next() else {
        return false;
    };
    sequences.all(|sequence| sequence.is_some_and(|dates| dates == reference))
}
