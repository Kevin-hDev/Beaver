use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_orchestration::ParentSubagentOrchestrator;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
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
}

pub(super) struct ApiRequestOutput {
    pub result: StreamResult,
    pub plan_active: bool,
    pub interrupted: bool,
}

pub(super) async fn run(params: ApiRequestParams<'_>) -> Result<ApiRequestOutput, String> {
    let completion_cancel = params.cancel.clone();
    params
        .subagents
        .prepare_for_model_request(params.messages)
        .await?;
    crate::services::agent_local::tool_result_budget::apply_budget(params.messages);
    crate::services::agent_local::context_budget::prepare_for_request(
        params.messages,
        params.configured_context,
    )?;
    let realtime_budget = RealtimeBudget::from_messages(params.configured_context, params.messages);
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
    let outcome = super::retry::retry_stream(
        params.on_event,
        params.session_id,
        params.request_id,
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
    .await?;
    let interrupted = outcome.is_interrupted();
    let result = outcome.into_result();
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
    })
}
