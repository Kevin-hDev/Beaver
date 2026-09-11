use super::agent_loop_request_types::ApiRequestParams;
use crate::services::agent_local::context_usage_buckets::RequestContextUsage;
use crate::services::agent_local::context_usage_runtime::ContextAttempt;
use crate::services::agent_local::context_usage_runtime::PreparedContextAttempt;
use crate::services::agent_local::types_ollama::StreamResult;

pub(super) async fn persist_result(
    params: &ApiRequestParams<'_>,
    attempt: u32,
    result: &StreamResult,
) -> Result<(), String> {
    context_attempt(params, attempt)
        .persist_result(result)
        .await
}

pub(super) fn prepared_attempt<'a>(
    params: &'a ApiRequestParams<'a>,
    attempt: u32,
    breakdown: RequestContextUsage,
) -> PreparedContextAttempt<'a> {
    PreparedContextAttempt::new(context_attempt(params, attempt), breakdown)
}

fn context_attempt<'a>(params: &'a ApiRequestParams<'a>, attempt: u32) -> ContextAttempt<'a> {
    ContextAttempt {
        on_event: params.on_event,
        journal: params.journal,
        provider_id: params.provider_id,
        model: params.model,
        turn: params.turn,
        attempt,
        context_limit: params.configured_context,
        measured_input_source:
            crate::services::agent_local::context_usage_record::ContextCountSource::Provider,
    }
}
