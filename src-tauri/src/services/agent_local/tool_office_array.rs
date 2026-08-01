use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrayInputError {
    Invalid,
    TooMany,
}

pub(super) fn coerce(value: &Value, max_items: usize) -> Result<Vec<Value>, ArrayInputError> {
    match value {
        Value::Array(values) => bounded_clone(values, max_items),
        Value::Object(_) if max_items > 0 => Ok(vec![value.clone()]),
        Value::String(encoded) => parse_encoded(encoded, max_items),
        _ => Err(ArrayInputError::Invalid),
    }
}

fn parse_encoded(encoded: &str, max_items: usize) -> Result<Vec<Value>, ArrayInputError> {
    if let Ok(parsed) = serde_json::from_str::<Value>(encoded.trim()) {
        return bounded_owned(parsed, max_items);
    }
    let unescaped = encoded.replace("\\\"", "\"").replace("\\\\", "\\");
    if unescaped == encoded {
        return Err(ArrayInputError::Invalid);
    }
    let parsed = serde_json::from_str::<Value>(unescaped.trim())
        .map_err(|_| ArrayInputError::Invalid)?;
    bounded_owned(parsed, max_items)
}

fn bounded_clone(values: &[Value], max_items: usize) -> Result<Vec<Value>, ArrayInputError> {
    if values.len() > max_items {
        return Err(ArrayInputError::TooMany);
    }
    Ok(values.to_vec())
}

fn bounded_owned(value: Value, max_items: usize) -> Result<Vec<Value>, ArrayInputError> {
    match value {
        Value::Array(values) if values.len() <= max_items => Ok(values),
        Value::Array(_) => Err(ArrayInputError::TooMany),
        Value::Object(_) if max_items > 0 => Ok(vec![value]),
        _ => Err(ArrayInputError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_and_encoded_arrays_share_the_same_limit() {
        let direct = serde_json::json!([{}, {}]);
        let encoded = Value::String("[{},{}]".into());

        assert_eq!(coerce(&direct, 1).unwrap_err(), ArrayInputError::TooMany);
        assert_eq!(coerce(&encoded, 1).unwrap_err(), ArrayInputError::TooMany);
    }
}
