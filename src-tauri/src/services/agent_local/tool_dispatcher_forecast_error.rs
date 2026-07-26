use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::selection_policy::ForecastSelectionMode;
use serde_json::Value;

pub(super) fn model_error(
    mode: Option<ForecastSelectionMode>,
    selected: &str,
    requested: Option<&str>,
    error: &str,
) -> ToolResult {
    let payload = model_error_payload(mode, selected, requested, error);
    serde_json::to_string_pretty(&payload)
        .map_or_else(|_| ToolResult::err(error), ToolResult::err)
}

pub(super) fn model_error_payload(
    mode: Option<ForecastSelectionMode>,
    selected: &str,
    requested: Option<&str>,
    error: &str,
) -> Value {
    let (mode, selector_locked, ignored, next_step) = match mode {
        Some(ForecastSelectionMode::Auto) => (
            "auto",
            false,
            None,
            "Corriger la requête. Si la sélection a expiré ou les ressources ont changé, relancer forecast_models avant forecast.",
        ),
        Some(ForecastSelectionMode::Manual) => (
            "manual",
            true,
            requested.filter(|model| *model != selected),
            "Corriger la requête. Le modèle reste imposé par le sélecteur Forecast.",
        ),
        None => (
            "unknown",
            false,
            None,
            "Corriger la requête puis relancer forecast.",
        ),
    };
    serde_json::json!({
        "error": error,
        "model_selection": {
            "mode": mode,
            "effective_model": selected,
            "requested_model_ignored": ignored,
            "selector_locked": selector_locked,
            "next_step": next_step
        }
    })
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_run_tests.rs"]
mod tests;
