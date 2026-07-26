use std::collections::BTreeMap;

use serde_json::Value;

use crate::services::forecast::types::ForecastResult;

#[derive(Clone, Copy)]
pub(super) struct Sample {
    pub delta: f64,
    pub value: f64,
}

pub(super) fn for_column(result: &ForecastResult, column: &str) -> (Vec<Sample>, Vec<Sample>) {
    let mut grouped = BTreeMap::<String, Vec<(f64, f64)>>::new();
    for row in &result.input_data.rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        let (Some(target), Some(value)) = (
            number(object.get(&result.target_column)),
            number(object.get(column)),
        ) else {
            continue;
        };
        let series = result
            .input_data
            .series_column
            .as_ref()
            .and_then(|name| object.get(name))
            .map(series_text)
            .unwrap_or_default();
        grouped.entry(series).or_default().push((target, value));
    }
    split_samples(grouped)
}

fn split_samples(grouped: BTreeMap<String, Vec<(f64, f64)>>) -> (Vec<Sample>, Vec<Sample>) {
    let mut training = Vec::new();
    let mut validation = Vec::new();
    for values in grouped.values() {
        let derived: Vec<_> = values
            .windows(2)
            .filter_map(|pair| {
                let sample = Sample {
                    delta: pair[1].0 - pair[0].0,
                    value: pair[1].1,
                };
                (sample.delta.is_finite() && sample.value.is_finite()).then_some(sample)
            })
            .collect();
        if derived.is_empty() {
            continue;
        }
        let split = (derived.len() * 7 / 10).clamp(1, derived.len());
        training.extend_from_slice(&derived[..split]);
        validation.extend_from_slice(&derived[split..]);
    }
    (training, validation)
}

fn number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64().filter(|value| value.is_finite()),
        Value::String(text) => {
            crate::services::forecast::numeric_parse::parse_finite_number(text).ok()
        }
        _ => None,
    }
}

fn series_text(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}
