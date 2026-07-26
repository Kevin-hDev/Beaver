use super::agent_loop_thinking_retry::{EagerHandle, ThinkingRetryParams};
use super::stream_events::AgentEventEmitter;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::{ChatMessage, OllamaThink, StreamResult};
use crate::services::compress::realtime_budget::RealtimeBudget;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct OllamaRequestParams<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub messages: &'a mut Vec<ChatMessage>,
    pub model: &'a str,
    pub tools: &'a [serde_json::Value],
    pub think: &'a OllamaThink,
    pub working_dir: &'a Path,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub configured_context: u64,
    pub plan_mode_active: bool,
    pub turn: usize,
    pub subagents: &'a mut ParentSubagentOrchestrator,
}

pub(super) struct OllamaRequestOutput {
    pub result: StreamResult,
    pub eager_handle: EagerHandle,
    pub plan_active: bool,
    pub interrupted: bool,
}

pub(super) async fn run(params: OllamaRequestParams<'_>) -> Result<OllamaRequestOutput, String> {
    let completion_cancel = params.cancel.clone();
    params
        .subagents
        .prepare_for_model_request(params.messages)
        .await?;
    super::session_security::sanitize_chat_messages(params.messages);
    super::tool_result_budget::apply_budget(params.messages);
    super::context_budget::prepare_for_request(params.messages, params.configured_context)?;
    let realtime_budget = RealtimeBudget::from_messages(params.configured_context, params.messages);
    let plan_active =
        super::agent_loop_plan::active(params.session_id, params.plan_mode_active).await;
    let request = super::agent_loop_support::build_request(
        params.model,
        params.messages,
        params.tools,
        params.think.clone(),
    );
    super::stream_diagnostics_model::record_model_request(
        params.session_id,
        params.request_id,
        params.turn,
        params.messages,
    )
    .await;
    super::stream_diagnostics_payload::record_ollama_payload(
        params.session_id,
        params.request_id,
        params.turn,
        &request,
    )
    .await;
    let (tool_tx, tool_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut eager_handle = tokio::spawn(super::eager_dispatch::collect_eager_results(
        tool_rx,
        params.working_dir.to_path_buf(),
        params.session_id.to_string(),
        params.request_id.to_string(),
    ));
    super::stream_diagnostics::mark_phase(
        params.session_id,
        params.request_id,
        "model_stream",
        "Stream modèle démarré.",
    )
    .await;
    let outcome = super::ollama_stream::stream_chat_with_tool_notify(
        params.on_event,
        &request,
        params.cancel.clone(),
        tool_tx,
        plan_active,
        realtime_budget.clone(),
    )
    .await?;
    let mut interrupted = outcome.is_interrupted();
    let mut result = outcome.into_result();
    super::stream_diagnostics_model::record_model_result(
        params.session_id,
        params.request_id,
        params.turn,
        &result,
    )
    .await;
    if !interrupted {
        let retry = super::agent_loop_thinking_retry::retry_if_needed(ThinkingRetryParams {
            on_event: params.on_event,
            request: &request,
            result,
            eager_handle,
            turn: params.turn,
            working_dir: params.working_dir.to_path_buf(),
            session_id: params.session_id.to_string(),
            request_id: params.request_id.to_string(),
            cancel: params.cancel.clone(),
            plan_active,
            realtime_budget,
        })
        .await?;
        result = retry.result;
        eager_handle = retry.eager_handle;
        interrupted = retry.interrupted;
    }
    params
        .subagents
        .complete_model_request(!interrupted, &completion_cancel, params.messages)
        .await?;
    Ok(OllamaRequestOutput {
        result,
        eager_handle,
        plan_active,
        interrupted,
    })
}
