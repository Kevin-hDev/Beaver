use super::agent_loop_thinking_retry::{EagerHandle, ThinkingRetryParams};
use super::context_usage_buckets::{ContextUsageSeed, RequestContextUsage};
use super::generation_metrics::GenerationAggregate;
use super::stream_events::AgentEventEmitter;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::{ChatMessage, OllamaThink, StreamResult};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::reasoning_continuity::contract::{ContinuationUse, ReplayTarget};
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};
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
    pub chat_mode: bool,
    pub turn: usize,
    pub subagents: &'a mut ParentSubagentOrchestrator,
    pub context_usage_seed: ContextUsageSeed,
    pub capture_reasoning: bool,
    pub live_replay_target:
        Option<&'a crate::services::reasoning_continuity::contract::ReplayTarget>,
    #[cfg(debug_assertions)]
    pub fixture_candidate:
        Option<&'a crate::services::reasoning_continuity::contract::ReplayTarget>,
    pub enable_eager_tools: bool,
}

pub(super) struct OllamaRequestOutput {
    pub result: StreamResult,
    pub eager_handle: EagerHandle,
    pub plan_active: bool,
    pub interrupted: bool,
    pub input_tokens: u32,
    pub generation: GenerationAggregate,
}

pub(super) async fn run(params: OllamaRequestParams<'_>) -> Result<OllamaRequestOutput, String> {
    let completion_cancel = params.cancel.clone();
    params
        .subagents
        .prepare_for_model_request(params.messages)
        .await?;
    super::session_security::sanitize_chat_messages(params.messages);
    super::tool_result_budget::apply_budget(params.messages);
    let report = super::context_budget::prepare_for_request(
        params.messages,
        params.configured_context,
        params.tools,
        "ollama",
    )?;
    super::context_budget::record_repairs(&report, params.session_id, params.request_id).await;
    let input_tokens = super::context_usage_runtime::emit_input(
        params.on_event,
        report.estimated_tokens,
        params.configured_context,
        RequestContextUsage::from_request(
            "ollama",
            params.messages,
            params.tools,
            params.context_usage_seed,
        ),
    );
    let realtime_budget = RealtimeBudget::for_session(
        params.session_id,
        params.configured_context,
        report.estimated_tokens,
    )
    .await;
    let plan_active =
        super::agent_loop_plan::active(params.session_id, params.plan_mode_active).await;
    let mut request = super::agent_loop_support::build_request(
        params.model,
        params.messages,
        params.tools,
        params.think.clone(),
    );
    request.capture_reasoning = params.capture_reasoning;
    request.live_replay_target = params
        .live_replay_target
        .map(|target| live_target_for_request(target, follows_tool_result(params.messages)))
        .transpose()?;
    #[cfg(debug_assertions)]
    {
        request.fixture_candidate = params.fixture_candidate.cloned();
    }
    if !request.capture_reasoning {
        crate::services::reasoning_continuity::diagnostics::record_blocked(
            params.session_id,
            params.request_id,
        )
        .await;
    }
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
    let mut eager_handle = super::agent_loop_thinking_retry::spawn_eager_handle(
        tool_rx,
        params.working_dir.to_path_buf(),
        params.session_id.to_string(),
        params.request_id.to_string(),
        params.chat_mode,
        params.cancel.clone(),
        params.enable_eager_tools,
    );
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
        super::ollama_stream_request::ReplayDiagnosticContext {
            session_id: params.session_id,
            request_id: params.request_id,
        },
    )
    .await?;
    let mut interrupted = outcome.is_interrupted();
    let mut result = outcome.into_result();
    let mut generation = GenerationAggregate::default();
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
            chat_mode: params.chat_mode,
            realtime_budget,
            enable_eager_tools: params.enable_eager_tools,
        })
        .await?;
        result = retry.result;
        eager_handle = retry.eager_handle;
        interrupted = retry.interrupted;
        generation = retry.generation;
    } else {
        generation.add_result(&result);
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
        input_tokens,
        generation,
    })
}

fn follows_tool_result(messages: &[ChatMessage]) -> bool {
    messages
        .last()
        .is_some_and(|message| message.role == "tool")
}

fn live_target_for_request(
    target: &ReplayTarget,
    follows_tool_result: bool,
) -> Result<ReplayTarget, String> {
    let mut target = target.clone();
    target.continuation_use = if follows_tool_result {
        ContinuationUse::ToolContinuation
    } else {
        ContinuationUse::UserContinuation
    };
    let allowed = crate::services::reasoning_continuity::registry::replay_policy(&target)
        .is_some_and(|policy| {
            policy.activation() == ActivationState::LiveValidated
                && policy.requirement() != ReplayRequirement::Forbidden
        });
    allowed
        .then_some(target)
        .ok_or_else(|| "reasoning_continuity_invalid".to_string())
}

#[cfg(test)]
#[path = "agent_loop_ollama_request_tests.rs"]
mod tests;
