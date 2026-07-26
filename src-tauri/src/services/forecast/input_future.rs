use super::input_data::ParsedInput;
use super::input_dates::build_future_dates;
use super::input_series::read_series_id;
use super::types::ForecastRequest;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn expected_dates_by_series(
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    if !input.future_rows.is_empty() {
        return dates_from_future_rows(request, input);
    }
    let mut last_dates = BTreeMap::<String, String>::new();
    for point in &input.snapshot.history {
        let id = point.series_id.as_deref().unwrap_or("series-1");
        last_dates.insert(id.to_string(), point.date.clone());
    }
    Ok(last_dates
        .into_iter()
        .map(|(id, date)| {
            (
                id,
                build_future_dates(&date, &request.frequency, request.horizon),
            )
        })
        .collect())
}

fn dates_from_future_rows(
    request: &ForecastRequest,
    input: &ParsedInput,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for row in &input.future_rows {
        let object = row.as_object().ok_or("Ligne future invalide")?;
        let id = read_series_id(object, request)?.unwrap_or_else(|| "series-1".into());
        let date = object
            .get(&request.date_column)
            .and_then(Value::as_str)
            .ok_or("Date future invalide")?;
        grouped.entry(id).or_default().push(date.to_string());
    }
    Ok(grouped)
}
