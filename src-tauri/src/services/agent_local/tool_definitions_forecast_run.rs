use crate::services::agent_local::tool_definitions;
use crate::services::forecast::selection_policy::{self, ForecastSelectionMode};
use serde_json::Value;

pub(super) fn definition() -> Value {
    let policy = selection_policy::get().unwrap_or_default();
    definition_for(policy)
}

fn definition_for(policy: selection_policy::ForecastSelectionPolicy) -> Value {
    let (policy_text, auto) = match policy.mode {
        ForecastSelectionMode::Manual => {
            let text = match policy.manual_model_id {
                Some(model) => format!(
                    "The Forecast selector forces '{model}'. After forecast_data_audit, call forecast_models to inspect this model's interval_capability. Run forecast only if it supports the exact audited confidence_level. Never round the user's request or modify this policy."
                ),
                None => "No manual Forecast model is selected. You must ask the user to select one before you run forecast.".to_string(),
            };
            (text, false)
        }
        ForecastSelectionMode::Auto => (
            "Auto is active. After forecast_data_audit, you must call forecast_models with the returned data_profile_id before the first forecast or after the task changes. Every returned candidate already supports the profile's exact confidence_level; pass that level unchanged. Choose only one returned candidate and pass its model id plus the returned selection_id. Prefer comparable rolling backtests when present. Set selection_source to auto, or explicit_user_override only when the user explicitly requested that safe candidate. Add only short allowlisted selection_reason_codes. If the backend reports an expired selection or changed resources, call forecast_models again. You must not call a capability-only choice the best model, round confidence, or modify the user's policy.".to_string(),
            true,
        ),
    };
    let mut properties = super::forecast_data::properties();
    if auto {
        properties.insert(
            "model".into(),
            serde_json::json!({
                "type": "string",
                "description": "Model id selected from the bounded forecast_models candidates list."
            }),
        );
        properties.insert(
            "selection_id".into(),
            serde_json::json!({
                "type": "string",
                "maxLength": 64,
                "description": "Short-lived selection id returned by forecast_models for this dataset and session."
            }),
        );
        properties.insert(
            "selection_source".into(),
            serde_json::json!({
                "type": "string",
                "enum": ["auto", "explicit_user_override"],
                "description": "Use explicit_user_override only for an explicit user model request that remained a safe candidate."
            }),
        );
        properties.insert(
            "selection_reason_codes".into(),
            serde_json::json!({
                "type": "array",
                "maxItems": crate::services::forecast::limits::MAX_SELECTION_REASON_CODES,
                "items": {
                    "type": "string",
                    "enum": [
                        "top_backtest", "beats_baseline", "precision_requested",
                        "speed_requested", "local_required", "cloud_allowed",
                        "user_requested", "resource_fit"
                    ]
                },
                "description": "Short structured reasons for the final safe choice."
            }),
        );
    }
    let mut required = vec![
        "target_column",
        "date_column",
        "horizon",
        "frequency",
        "confidence_level",
    ];
    if auto {
        required.push("model");
        required.push("selection_id");
        required.push("selection_source");
    }
    let description = format!(
        "Run a validated time series forecast. For every new dataset, you must call forecast_data_audit first and then pass its reusable data_profile_id plus the exact same confidence_level instead of resending raw data. Direct data or file_path remains available for application compatibility. The tool returns a saved analysis_id; you call forecast_read with it for paginated predictions and quantiles. {policy_text} You use series_column for multi-series data and covariate_columns only when the selected model supports them."
    );
    tool_definitions::tool_def(
        "forecast",
        &description,
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        }),
    )
}

#[cfg(test)]
#[path = "tool_definitions_forecast_run_tests.rs"]
mod tests;
