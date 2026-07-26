use crate::services::forecast::input_parse_utils::read_target_value;
use crate::services::forecast::input_series::normalize_series_value;
use crate::services::forecast::types::{ForecastResult, Prediction};
use serde_json::Value;

#[derive(Debug, Clone)]
pub(super) struct SourcePoint {
    pub row: Value,
    pub date: String,
    pub series_id: Option<String>,
    pub value: f64,
}

pub(super) fn load(analysis: &ForecastResult) -> Result<Vec<SourcePoint>, String> {
    if analysis.input_data.rows.is_empty() {
        return Ok(analysis
            .input_data
            .history
            .iter()
            .map(|point| SourcePoint {
                row: fallback_row(analysis, point),
                date: point.date.clone(),
                series_id: point.series_id.clone(),
                value: point.value,
            })
            .collect());
    }
    let date_column = analysis
        .input_data
        .date_column
        .as_deref()
        .ok_or("Données historiques incompatibles")?;
    let mut points = Vec::new();
    for row in &analysis.input_data.rows {
        let object = row.as_object().ok_or("Données historiques incompatibles")?;
        let Some(value) = read_target_value(object.get(&analysis.target_column))? else {
            continue;
        };
        let date = object
            .get(date_column)
            .and_then(Value::as_str)
            .ok_or("Données historiques incompatibles")?
            .to_string();
        let series_id = match analysis.input_data.series_column.as_deref() {
            Some(column) => normalize_series_value(
                object
                    .get(column)
                    .ok_or("Données historiques incompatibles")?,
            )?,
            None => None,
        };
        points.push(SourcePoint {
            row: row.clone(),
            date,
            series_id,
            value,
        });
    }
    Ok(points)
}

fn fallback_row(analysis: &ForecastResult, point: &Prediction) -> Value {
    let date_column = analysis.input_data.date_column.as_deref().unwrap_or("date");
    let mut row = serde_json::Map::new();
    row.insert(date_column.to_string(), Value::String(point.date.clone()));
    row.insert(
        analysis.target_column.clone(),
        serde_json::json!(point.value),
    );
    if let (Some(column), Some(id)) = (&analysis.input_data.series_column, &point.series_id) {
        row.insert(column.clone(), Value::String(id.clone()));
    }
    Value::Object(row)
}
