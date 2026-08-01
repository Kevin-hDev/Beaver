use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::forecast::selection_policy::ForecastSelectionMode;
use serde_json::Value;

pub(super) enum ForecastErrorKind {
    Validation(&'static str),
    NotFound(&'static str),
    Unavailable(&'static str, bool),
    External(&'static str, bool),
    Internal(&'static str, bool),
}

impl ForecastErrorKind {
    fn parts(self) -> (&'static str, ToolErrorCategory, bool) {
        match self {
            Self::Validation(code) => (code, ToolErrorCategory::Validation, false),
            Self::NotFound(code) => (code, ToolErrorCategory::NotFound, false),
            Self::Unavailable(code, retryable) => {
                (code, ToolErrorCategory::Unavailable, retryable)
            }
            Self::External(code, retryable) => (code, ToolErrorCategory::External, retryable),
            Self::Internal(code, retryable) => (code, ToolErrorCategory::Internal, retryable),
        }
    }
}

pub(super) fn model_error(
    kind: ForecastErrorKind,
    mode: Option<ForecastSelectionMode>,
    selected: &str,
    requested: Option<&str>,
    error: &str,
) -> ToolResult {
    let payload = model_error_payload(mode, selected, requested, error);
    let content = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| error.to_string());
    let (code, category, retryable) = kind.parts();
    ToolResult::error(content, code, category, retryable)
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
