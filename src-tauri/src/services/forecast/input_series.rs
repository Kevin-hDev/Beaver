use super::types::ForecastRequest;
use serde_json::{Map, Value};

pub fn read_series_id(
    row: &Map<String, Value>,
    request: &ForecastRequest,
) -> Result<Option<String>, String> {
    let Some(column) = request.series_column.as_ref() else {
        return Ok(None);
    };
    let value = row.get(column).ok_or("Colonne série introuvable")?;
    normalize_series_value(value)
}

pub fn normalize_series_value(value: &Value) -> Result<Option<String>, String> {
    match value {
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err("Valeur de série invalide".into());
            }
            Ok(Some(trimmed.to_string()))
        }
        Value::Number(number) => Ok(Some(number.to_string())),
        Value::Bool(flag) => Ok(Some(flag.to_string())),
        _ => Err("Valeur de série invalide".into()),
    }
}
