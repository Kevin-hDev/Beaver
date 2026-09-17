use serde_json::{Map, Value};

pub(super) fn properties() -> Map<String, Value> {
    let limits = crate::services::forecast::limits::ToolSchemaLimits::default();
    serde_json::from_value(serde_json::json!({
        "data": {
            "type": "string",
            "maxLength": limits.inline_data_chars,
            "description": "JSON row array. Historical rows include date and target; future-known rows may omit target."
        },
        "file_path": {
            "type": "string",
            "maxLength": limits.path_chars,
            "description": "CSV or Excel path used instead of data."
        },
        "data_profile_id": {
            "type": "string",
            "maxLength": limits.id_chars,
            "description": "Reusable id returned by forecast_data_audit. Use it instead of data or file_path."
        },
        "target_column": {"type": "string", "maxLength": limits.column_chars, "description": "Target column name."},
        "date_column": {"type": "string", "maxLength": limits.column_chars, "description": "Date or timestamp column name."},
        "series_column": {"type": "string", "maxLength": limits.column_chars, "description": "Optional series identifier column."},
        "covariate_columns": {
            "type": "array",
            "maxItems": limits.covariates,
            "items": {"type": "string", "maxLength": limits.column_chars},
            "description": "Optional compatible past or future-known context columns."
        },
        "horizon": {"type": "integer", "minimum": 1, "maximum": limits.horizon, "description": "Future steps to predict."},
        "frequency": {"type": "string", "maxLength": limits.frequency_chars, "description": "Frequency such as D, B, W, M, Q, Y, H, or T."},
        "confidence_level": {
            "type": "number",
            "minimum": 0.5,
            "maximum": 0.99,
            "description": "Use the user's exact requested prediction-interval confidence. If none was requested, use 0.8, the common safe default. Use whole percentage steps only and never round an explicit request to fit a model."
        }
    }))
    .unwrap_or_default()
}
