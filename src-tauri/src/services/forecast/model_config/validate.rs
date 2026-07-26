use super::schema::{ParamKind, ParamSpec};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

const MAX_TEXT_LEN: usize = 80;
const MAX_NUMBER_LIST: usize = 9;

pub fn sanitize(
    specs: &[ParamSpec],
    values: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let by_id: BTreeMap<&str, &ParamSpec> = specs.iter().map(|spec| (spec.id, spec)).collect();
    let mut clean = Map::new();
    for (id, value) in values {
        let Some(spec) = by_id.get(id.as_str()) else {
            return Err("Paramètre Forecast invalide".into());
        };
        if should_remove(&value) {
            continue;
        }
        let normalized = normalize_value(spec, value)?;
        if normalized != spec.default_value {
            clean.insert(id, normalized);
        }
    }
    Ok(clean)
}

fn should_remove(value: &Value) -> bool {
    value.is_null()
        || value.as_str().is_some_and(|text| text.trim().is_empty())
        || value.as_array().is_some_and(|items| items.is_empty())
}

fn normalize_value(spec: &ParamSpec, value: Value) -> Result<Value, String> {
    match spec.kind {
        ParamKind::Integer => normalize_integer(spec, value),
        ParamKind::Number => normalize_number(spec, value),
        ParamKind::Boolean => value
            .as_bool()
            .map(Value::Bool)
            .ok_or_else(|| "Paramètre Forecast invalide".to_string()),
        ParamKind::Select => normalize_select(spec, value),
        ParamKind::NumberList => normalize_number_list(spec, value),
    }
}

fn normalize_integer(spec: &ParamSpec, value: Value) -> Result<Value, String> {
    let number = value
        .as_i64()
        .or_else(|| value.as_str()?.trim().parse::<i64>().ok())
        .ok_or_else(|| "Paramètre Forecast invalide".to_string())?;
    validate_range(spec, number as f64)?;
    Ok(Value::Number(number.into()))
}

fn normalize_number(spec: &ParamSpec, value: Value) -> Result<Value, String> {
    let number = value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse::<f64>().ok())
        .filter(|number| number.is_finite())
        .ok_or_else(|| "Paramètre Forecast invalide".to_string())?;
    validate_range(spec, number)?;
    serde_json::Number::from_f64(number)
        .map(Value::Number)
        .ok_or_else(|| "Paramètre Forecast invalide".to_string())
}

fn normalize_select(spec: &ParamSpec, value: Value) -> Result<Value, String> {
    let text = value
        .as_str()
        .map(str::trim)
        .filter(|text| text.len() <= MAX_TEXT_LEN)
        .ok_or_else(|| "Paramètre Forecast invalide".to_string())?;
    if !spec.options.iter().any(|option| option == &text) {
        return Err("Paramètre Forecast invalide".into());
    }
    Ok(Value::String(text.to_string()))
}

fn normalize_number_list(spec: &ParamSpec, value: Value) -> Result<Value, String> {
    let values = if let Some(items) = value.as_array() {
        items.iter().filter_map(Value::as_f64).collect::<Vec<_>>()
    } else if let Some(text) = value.as_str() {
        text.split(',')
            .filter_map(|item| item.trim().parse::<f64>().ok())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if values.is_empty() || values.len() > MAX_NUMBER_LIST {
        return Err("Paramètre Forecast invalide".into());
    }
    let mut values = values;
    values.sort_by(|left, right| left.total_cmp(right));
    values.dedup_by(|left, right| (*left - *right).abs() < f64::EPSILON);
    let mut normalized = Vec::with_capacity(values.len());
    for number in values {
        if !number.is_finite() {
            return Err("Paramètre Forecast invalide".into());
        }
        validate_range(spec, number)?;
        let json_number = serde_json::Number::from_f64(number)
            .ok_or_else(|| "Paramètre Forecast invalide".to_string())?;
        normalized.push(Value::Number(json_number));
    }
    Ok(Value::Array(normalized))
}

fn validate_range(spec: &ParamSpec, value: f64) -> Result<(), String> {
    if spec.min.is_some_and(|min| value < min) || spec.max.is_some_and(|max| value > max) {
        return Err("Paramètre Forecast hors limites".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "validate_collection_tests.rs"]
mod collection_tests;
#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
