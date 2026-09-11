use super::agent_loop_ollama_request::OllamaRequestParams;
use super::context_usage_buckets::RequestContextUsage;
use super::types_ollama::StreamResult;

pub(super) async fn persist_preparation(
    params: &OllamaRequestParams<'_>,
    attempt: u32,
    input_tokens: usize,
    breakdown: RequestContextUsage,
) -> Result<u32, String> {
    super::context_usage_runtime::ContextAttempt {
        on_event: params.on_event,
        journal: params.journal,
        provider_id: "ollama",
        model: params.model,
        turn: params.turn,
        attempt,
        context_limit: params.configured_context,
        measured_input_source:
            super::context_usage_record::ContextCountSource::NativeCounter,
    }
    .persist_preparation(input_tokens, breakdown)
    .await
}

pub(super) async fn persist_result(
    params: &OllamaRequestParams<'_>,
    attempt: u32,
    result: &StreamResult,
) -> Result<(), String> {
    super::context_usage_runtime::ContextAttempt {
        on_event: params.on_event,
        journal: params.journal,
        provider_id: "ollama",
        model: params.model,
        turn: params.turn,
        attempt,
        context_limit: params.configured_context,
        measured_input_source:
            super::context_usage_record::ContextCountSource::NativeCounter,
    }
    .persist_result(result)
    .await
}
