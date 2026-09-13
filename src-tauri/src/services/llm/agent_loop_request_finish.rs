use super::agent_loop_request_types::{ApiRequestOutput, ApiRequestParams};
use crate::services::agent_local::generation_metrics::GenerationAggregate;
use crate::services::agent_local::types_ollama::StreamOutcome;
use tokio_util::sync::CancellationToken;

pub(super) async fn finish(
    params: ApiRequestParams<'_>,
    outcome: StreamOutcome,
    completed_attempt: u32,
    plan_active: bool,
    completion_cancel: CancellationToken,
) -> Result<ApiRequestOutput, String> {
    let interrupted = outcome.is_interrupted();
    let result = outcome.into_result();
    super::agent_loop_request_context::persist_result(&params, completed_attempt, &result).await?;
    let mut generation = GenerationAggregate::default();
    generation.add_result(&result);
    crate::services::provider_usage::record_for_session(
        params.provider_id,
        params.model,
        params.session_id,
        crate::services::provider_usage::UsageWorkload::Primary,
        result.usage.as_ref(),
    )
    .await;
    crate::services::agent_local::stream_diagnostics_model::record_model_result(
        params.session_id,
        params.request_id,
        params.turn,
        &result,
    )
    .await;
    params
        .subagents
        .complete_model_request(
            !interrupted && result.completion_error.is_none(),
            &completion_cancel,
            params.messages,
        )
        .await?;
    Ok(ApiRequestOutput {
        result,
        plan_active,
        interrupted,
        generation,
    })
}
