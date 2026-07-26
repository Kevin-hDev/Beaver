use super::schema::{ParamKind, ParamSpec};
use serde_json::{json, Value};

pub(super) fn horizon_override() -> ParamSpec {
    int_param(
        "horizon_max_override",
        0,
        0.0,
        crate::services::forecast::limits::MAX_HORIZON as f64,
    )
}

pub(super) fn context() -> ParamSpec {
    int_param("context_length", 0, 0.0, 100_000.0)
}

pub(super) fn batch_size() -> ParamSpec {
    int_param("batch_size", 1, 1.0, 1024.0)
}

pub(super) fn quantiles() -> ParamSpec {
    ParamSpec {
        id: "quantiles",
        kind: ParamKind::NumberList,
        default_value: json!([0.1, 0.5, 0.9]),
        min: Some(0.01),
        max: Some(0.99),
        options: &[],
    }
}

pub(super) fn dtype() -> ParamSpec {
    select("dtype", "auto", &["auto", "float32", "float16", "bfloat16"])
}

pub(super) fn bool_param(id: &'static str, default_value: bool) -> ParamSpec {
    ParamSpec {
        id,
        kind: ParamKind::Boolean,
        default_value: json!(default_value),
        min: None,
        max: None,
        options: &[],
    }
}

pub(super) fn int_param(id: &'static str, default_value: i64, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        id,
        kind: ParamKind::Integer,
        default_value: json!(default_value),
        min: Some(min),
        max: Some(max),
        options: &[],
    }
}

pub(super) fn number_param(id: &'static str, default_value: f64, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        id,
        kind: ParamKind::Number,
        default_value: json!(default_value),
        min: Some(min),
        max: Some(max),
        options: &[],
    }
}

pub(super) fn select(
    id: &'static str,
    default_value: &'static str,
    options: &'static [&'static str],
) -> ParamSpec {
    ParamSpec {
        id,
        kind: ParamKind::Select,
        default_value: Value::String(default_value.to_string()),
        min: None,
        max: None,
        options,
    }
}
