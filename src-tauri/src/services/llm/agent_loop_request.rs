use super::agent_loop_request_types::ApiRequestOutput;
use crate::services::agent_local::context_usage_buckets::{ContextUsageSeed, RequestContextUsage};
use crate::services::agent_local::generation_metrics::GenerationAggregate;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_orchestration::ParentSubagentOrchestrator;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::realtime_budget::RealtimeBudget;
use tokio_util::sync::CancellationToken;

pub(super) struct ApiRequestParams<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub provider_id: &'a str,
    pub model: &'a str,
    pub messages: &'a mut Vec<ChatMessage>,
    pub tools: &'a [serde_json::Value],
    pub think: bool,
    pub reasoning_mode: Option<&'a str>,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub configured_context: u64,
    pub plan_mode_active: bool,
    pub turn: usize,
    pub subagents: &'a mut ParentSubagentOrchestrator,
    pub context_usage_seed: ContextUsageSeed,
}

pub(super) async fn run(params: ApiRequestParams<'_>) -> Result<ApiRequestOutput, String> {
    let completion_cancel = params.cancel.clone();
    params
        .subagents
        .prepare_for_model_request(params.messages)
        .await?;
    crate::services::agent_local::tool_result_budget::apply_budget(params.messages);
    let report = crate::services::agent_local::context_budget::prepare_for_request(
        params.messages,
        params.configured_context,
        params.tools,
        params.provider_id,
    )?;
    crate::services::agent_local::context_budget::record_repairs(
        &report,
        params.session_id,
        params.request_id,
    )
    .await;
    let mut input_estimate = report.estimated_tokens;
    let mut input_tokens = crate::services::agent_local::context_usage_runtime::emit_input(
        params.on_event,
        input_estimate,
        params.configured_context,
        RequestContextUsage::from_request(
            params.provider_id,
            params.messages,
            params.tools,
            params.context_usage_seed,
        ),
    );
    let realtime_budget = RealtimeBudget::from_estimate(params.configured_context, input_estimate);
    let plan_active = crate::services::agent_local::agent_loop_plan::active(
        params.session_id,
        params.plan_mode_active,
    )
    .await;
    crate::services::agent_local::stream_diagnostics_model::record_model_request(
        params.session_id,
        params.request_id,
        params.turn,
        params.messages,
    )
    .await;
    crate::services::agent_local::stream_diagnostics_payload::record_api_payload(
        params.session_id,
        params.request_id,
        params.turn,
        params.provider_id,
        params.messages,
    )
    .await;
    crate::services::agent_local::stream_diagnostics::mark_phase(
        params.session_id,
        params.request_id,
        "model_stream",
        "Stream modèle démarré.",
    )
    .await;
    let purpose =
        crate::services::llm::request_purpose::RequestPurpose::for_session(params.session_id).await;
    let mut next_attempt = 1_u32;
    let turn = params.turn.try_into().unwrap_or(u32::MAX);
    let first_attempt = super::retry::retry_stream(
        params.on_event,
        params.session_id,
        params.request_id,
        turn,
        &mut next_attempt,
        params.provider_id,
        purpose,
        params.model,
        params.messages,
        params.tools,
        params.think,
        params.reasoning_mode,
        params.cancel.clone(),
        plan_active,
        realtime_budget,
    )
    .await;
    let outcome = match first_attempt {
        Ok(outcome) => outcome,
        Err(error) if error == "provider_payload_too_large" => {
            let changed =
                crate::services::agent_local::context_budget::reduce_after_payload_too_large(
                    params.messages,
                    params.configured_context,
                    params.tools,
                    params.provider_id,
                )?;
            if !changed {
                return Err(error);
            }
            let reduced_report = crate::services::agent_local::context_budget::prepare_for_request(
                params.messages,
                params.configured_context,
                params.tools,
                params.provider_id,
            )?;
            input_estimate = reduced_report.estimated_tokens;
            input_tokens = crate::services::agent_local::context_usage_runtime::emit_input(
                params.on_event,
                input_estimate,
                params.configured_context,
                RequestContextUsage::from_request(
                    params.provider_id,
                    params.messages,
                    params.tools,
                    params.context_usage_seed,
                ),
            );
            crate::services::agent_local::stream_diagnostics::record_retry(
                params.session_id,
                params.request_id,
                "Requête provider réduite après un rejet de taille.",
            )
            .await;
            let reduced_budget =
                RealtimeBudget::from_estimate(params.configured_context, input_estimate);
            super::retry::retry_stream(
                params.on_event,
                params.session_id,
                params.request_id,
                turn,
                &mut next_attempt,
                params.provider_id,
                purpose,
                params.model,
                params.messages,
                params.tools,
                params.think,
                params.reasoning_mode,
                params.cancel.clone(),
                plan_active,
                reduced_budget,
            )
            .await?
        }
        Err(error) => return Err(error),
    };
    let interrupted = outcome.is_interrupted();
    let result = outcome.into_result();
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
        .complete_model_request(!interrupted, &completion_cancel, params.messages)
        .await?;
    Ok(ApiRequestOutput {
        result,
        plan_active,
        interrupted,
        input_tokens,
        generation,
    })
}
