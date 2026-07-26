use super::client_local_response::ParsedPrediction;
use super::client_quantiles;
use super::input_data::ParsedInput;
use super::types::{ForecastRequest, Prediction};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub fn parse(
    body: &Value,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<ParsedPrediction, String> {
    let items = body["predictions"]
        .as_array()
        .ok_or("Réponse Forecast: champ predictions manquant")?;
    let expected_dates = super::input_future::expected_dates_by_series(request, input)?;
    let mut offsets = HashMap::<String, usize>::new();
    let mut predictions = Vec::with_capacity(items.len());
    let mut lower = Vec::with_capacity(items.len());
    let mut middle = Vec::with_capacity(items.len());
    let mut upper = Vec::with_capacity(items.len());

    for item in items {
        let raw_date = item["date"]
            .as_str()
            .ok_or("Réponse Forecast: date manquante")?;
        let value = item["value"]
            .as_f64()
            .filter(|number| number.is_finite())
            .ok_or("Réponse Forecast: valeur manquante")?;
        let series_id = output_series_id(item, request);
        let key = series_id.as_deref().unwrap_or("series-1");
        let offset = offsets.entry(key.to_string()).or_default();
        let date = output_date(raw_date, key, *offset, &expected_dates)?;
        *offset += 1;
        predictions.push(Prediction {
            date,
            value,
            series_id,
        });
        lower.push(bound_value(
            item,
            super::intervals::lower_level(request.confidence_level),
        )?);
        middle.push(middle_value(item, value)?);
        upper.push(bound_value(
            item,
            super::intervals::upper_level(request.confidence_level),
        )?);
    }
    Ok((predictions, lower, middle, upper))
}

fn middle_value(item: &Value, fallback: f64) -> Result<f64, String> {
    let Some(raw) = item.get("q50") else {
        return Ok(fallback);
    };
    raw.as_f64()
        .filter(|value| value.is_finite())
        .ok_or("Médiane Forecast invalide".into())
}

fn bound_value(item: &Value, level: f64) -> Result<f64, String> {
    client_quantiles::value_at_level(item, level).ok_or("Intervalle Forecast incomplet".into())
}

fn output_series_id(item: &Value, request: &ForecastRequest) -> Option<String> {
    request
        .series_column
        .as_ref()
        .and_then(|_| item["series_id"].as_str().map(str::to_string))
}

fn output_date(
    raw_date: &str,
    series_id: &str,
    index: usize,
    expected_dates: &BTreeMap<String, Vec<String>>,
) -> Result<String, String> {
    let expected = expected_dates
        .get(series_id)
        .and_then(|dates| dates.get(index))
        .ok_or("Date de sortie invalide")?;
    if raw_date.trim().to_ascii_uppercase().starts_with("T+") {
        return Ok(expected.clone());
    }
    if raw_date == expected {
        return Ok(expected.clone());
    }
    let raw_datetime = super::input_dates::parse_input_datetime(raw_date);
    let expected_datetime = super::input_dates::parse_input_datetime(expected);
    if raw_datetime.is_some() && raw_datetime == expected_datetime {
        return Ok(expected.clone());
    }
    Err("Date de sortie invalide".into())
}

#[cfg(test)]
#[path = "client_local_structured_tests.rs"]
mod tests;
