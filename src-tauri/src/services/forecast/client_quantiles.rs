use serde_json::Value;

pub fn array_at_level(body: &Value, level: f64) -> Vec<f64> {
    exact_array(
        body,
        &crate::services::forecast::intervals::quantile_key(level),
    )
    .unwrap_or_default()
}

pub fn value_at_level(item: &Value, level: f64) -> Option<f64> {
    item[crate::services::forecast::intervals::quantile_key(level)]
        .as_f64()
        .filter(|value| value.is_finite())
}

fn exact_array(body: &Value, key: &str) -> Option<Vec<f64>> {
    let values = body[key]
        .as_array()?
        .iter()
        .map(|value| value.as_f64().filter(|number| number.is_finite()))
        .collect::<Option<Vec<_>>>()?;
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_only_the_requested_confidence_bound() {
        let body = json!({ "q05": [1.0], "q10": [2.0], "q95": [9.0] });
        assert_eq!(array_at_level(&body, 0.05), vec![1.0]);
        assert_eq!(array_at_level(&body, 0.95), vec![9.0]);
    }

    #[test]
    fn rejects_non_finite_or_missing_arrays() {
        assert!(array_at_level(&json!({ "q05": ["bad"] }), 0.05).is_empty());
        assert!(array_at_level(&json!({}), 0.05).is_empty());
    }
}
