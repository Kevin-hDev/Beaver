use super::agent_loop_ollama_request::OllamaRequestParams;
use super::context_usage_buckets::RequestContextUsage;
use super::types_ollama::StreamResult;

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
        measured_input_source: super::context_usage_record::ContextCountSource::NativeCounter,
    }
    .persist_result(result)
    .await
}

pub(super) fn prepared_attempt<'a>(
    params: &'a OllamaRequestParams<'a>,
    attempt: u32,
    breakdown: RequestContextUsage,
) -> super::context_usage_runtime::PreparedContextAttempt<'a> {
    let baseline = crate::services::compress::prepared_request::count(
        "ollama",
        params.model,
        params.messages,
        params.tools,
    );
    super::context_usage_runtime::PreparedContextAttempt::new(
        super::context_usage_runtime::ContextAttempt {
            on_event: params.on_event,
            journal: params.journal,
            provider_id: "ollama",
            model: params.model,
            turn: params.turn,
            attempt,
            context_limit: params.configured_context,
            measured_input_source: super::context_usage_record::ContextCountSource::NativeCounter,
        },
        breakdown,
    )
    .with_baseline_count(baseline)
}
