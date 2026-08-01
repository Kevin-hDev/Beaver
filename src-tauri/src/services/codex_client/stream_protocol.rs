use super::http_error;
use crate::services::compress::realtime_budget::RealtimeBudget;

pub(super) fn closed_before_completed() -> String {
    "provider_connection_failed".to_string()
}

pub(super) fn incomplete_response() -> String {
    "provider_temporarily_unavailable".to_string()
}

pub(super) fn failed_response(event: &serde_json::Value) -> String {
    http_error::stream_failure(event)
}

pub(super) fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_tool_call: bool,
) -> bool {
    !has_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}

#[cfg(test)]
#[path = "stream_protocol_tests.rs"]
mod tests;
