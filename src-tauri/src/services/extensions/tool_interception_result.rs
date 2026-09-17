use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(tag = "decision", rename_all = "camelCase", deny_unknown_fields)]
enum InterceptorDecision {
    Continue,
    Deny { reason: Option<String> },
    Invalid,
    Failed,
}

pub(super) enum CallError {
    Timeout,
    Cancelled,
    Ineligible,
    Failed,
}

pub(super) enum Outcome {
    Continue,
    Deny,
    ChainTimeout,
    Disable(&'static str),
    Cancelled,
}

pub(super) fn classify(
    response: Result<Value, CallError>,
    handler_deadline: tokio::time::Instant,
    chain_deadline: tokio::time::Instant,
) -> Outcome {
    let now = tokio::time::Instant::now();
    if now >= chain_deadline || now >= handler_deadline {
        return timeout_outcome(handler_deadline, chain_deadline);
    }
    match response {
        Ok(value) => match serde_json::from_value::<InterceptorDecision>(value) {
            Ok(InterceptorDecision::Continue) => Outcome::Continue,
            Ok(InterceptorDecision::Deny { reason }) => {
                let _ = reason;
                Outcome::Deny
            }
            Ok(InterceptorDecision::Invalid) | Err(_) => {
                Outcome::Disable(super::types::DIAGNOSTIC_INTERCEPTOR_INVALID)
            }
            Ok(InterceptorDecision::Failed) => {
                Outcome::Disable(super::types::DIAGNOSTIC_INTERCEPTOR_FAILED)
            }
        },
        Err(CallError::Cancelled) => Outcome::Cancelled,
        Err(CallError::Ineligible) => Outcome::Continue,
        Err(CallError::Failed) => Outcome::Disable(super::types::DIAGNOSTIC_INTERCEPTOR_FAILED),
        Err(CallError::Timeout) => timeout_outcome(handler_deadline, chain_deadline),
    }
}

fn timeout_outcome(
    handler_deadline: tokio::time::Instant,
    chain_deadline: tokio::time::Instant,
) -> Outcome {
    if chain_deadline <= handler_deadline {
        Outcome::ChainTimeout
    } else {
        Outcome::Disable(super::types::DIAGNOSTIC_INTERCEPTOR_TIMEOUT)
    }
}

pub(super) fn denied() -> ToolResult {
    ToolResult::error(
        "Une extension a refusé cette action.",
        super::types::backend_error_codes::PERMISSION_DENIED,
        ToolErrorCategory::Permission,
        false,
    )
}

pub(super) fn individual_failure() -> ToolResult {
    ToolResult::error(
        "Une extension n'a pas pu inspecter cette action.",
        super::types::backend_error_codes::INTERCEPTION_INDIVIDUAL_TIMEOUT,
        ToolErrorCategory::Timeout,
        false,
    )
}

pub(super) fn chain_timeout() -> ToolResult {
    ToolResult::error(
        "L'inspection de cette action a pris trop de temps.",
        super::types::backend_error_codes::INTERCEPTION_CHAIN_TIMEOUT,
        ToolErrorCategory::Timeout,
        false,
    )
}
