use super::input_data::ParsedInput;
use super::input_dates::parse_input_datetime;
use super::limits::{validate_prediction_budget, MAX_PREDICTIONS};
use super::types::{ForecastRequest, ForecastResult, Prediction};
use std::collections::{BTreeMap, BTreeSet};

pub fn validate(
    result: &ForecastResult,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<(), String> {
    let series_count = input.data_profile.series_count;
    let expected = validate_prediction_budget(series_count, request.horizon)?;
    if result.horizon != request.horizon
        || result.predictions.len() != expected
        || result.predictions.len() > MAX_PREDICTIONS
    {
        return Err("Réponse Forecast incomplète".into());
    }
    validate_quantiles(result)?;
    validate_series(result, request, input)?;
    if super::target_domain::requires_non_negative_output(request, input)
        && has_negative_output(result)
    {
        return Err("Réponse Forecast hors domaine".into());
    }
    Ok(())
}

fn validate_quantiles(result: &ForecastResult) -> Result<(), String> {
    let count = result.predictions.len();
    if result.quantiles.q10.len() != count
        || result.quantiles.q50.len() != count
        || result.quantiles.q90.len() != count
    {
        return Err("Intervalles de prédiction incomplets".into());
    }
    for index in 0..count {
        let point = result.predictions[index].value;
        let lower = result.quantiles.q10[index];
        let middle = result.quantiles.q50[index];
        let upper = result.quantiles.q90[index];
        if ![point, lower, middle, upper]
            .iter()
            .all(|value| value.is_finite())
            || lower > middle
            || middle > upper
            || point < lower
            || point > upper
        {
            return Err("Intervalles de prédiction invalides".into());
        }
    }
    Ok(())
}

pub(crate) fn validate_stored_quantiles(result: &ForecastResult) -> Result<(), String> {
    validate_quantiles(result)
}

fn validate_series(
    result: &ForecastResult,
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<(), String> {
    let mut grouped = BTreeMap::<String, Vec<&Prediction>>::new();
    for point in &result.predictions {
        let id = point.series_id.as_deref().unwrap_or("series-1").to_string();
        grouped.entry(id).or_default().push(point);
    }
    let expected_ids: BTreeSet<String> = if request.series_column.is_some() {
        input.snapshot.series_ids.iter().cloned().collect()
    } else {
        BTreeSet::from(["series-1".to_string()])
    };
    if grouped.keys().cloned().collect::<BTreeSet<_>>() != expected_ids {
        return Err("Séries de sortie invalides".into());
    }
    let expected_dates = expected_future_dates(request, input)?;
    for (id, points) in grouped {
        if points.len() != request.horizon as usize {
            return Err("Horizon de sortie invalide".into());
        }
        validate_dates(&id, &points, &expected_dates)?;
    }
    Ok(())
}

fn validate_dates(
    series_id: &str,
    points: &[&Prediction],
    expected_dates: &BTreeMap<String, Vec<chrono::NaiveDateTime>>,
) -> Result<(), String> {
    let parsed: Vec<_> = points
        .iter()
        .map(|point| parse_input_datetime(&point.date).ok_or("Date de sortie invalide".to_string()))
        .collect::<Result<_, _>>()?;
    let expected = expected_dates
        .get(series_id)
        .ok_or("Série de dates future manquante")?;
    if &parsed != expected {
        return Err("Dates de sortie incohérentes".into());
    }
    Ok(())
}

fn expected_future_dates(
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<BTreeMap<String, Vec<chrono::NaiveDateTime>>, String> {
    super::input_future::expected_dates_by_series(request, input)?
        .into_iter()
        .map(|(id, dates)| {
            let parsed = dates
                .iter()
                .map(|date| parse_input_datetime(date).ok_or("Dates futures invalides".to_string()))
                .collect::<Result<_, _>>()?;
            Ok((id, parsed))
        })
        .collect()
}

fn has_negative_output(result: &ForecastResult) -> bool {
    result.predictions.iter().any(|point| point.value < 0.0)
        || result.quantiles.q10.iter().any(|value| *value < 0.0)
        || result.quantiles.q50.iter().any(|value| *value < 0.0)
        || result.quantiles.q90.iter().any(|value| *value < 0.0)
}

#[cfg(test)]
#[path = "result_validation_tests.rs"]
mod tests;
