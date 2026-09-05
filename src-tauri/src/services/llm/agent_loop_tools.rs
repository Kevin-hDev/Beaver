use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::circuit_breaker::CircuitBreaker;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_execution_outcome::ToolExecutionOutcome;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::write_guard::WriteGuard;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct ToolTurnContext<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub messages: &'a mut Vec<ChatMessage>,
    pub result: &'a crate::services::agent_local::types_ollama::StreamResult,
    pub working_dir: &'a Path,
    pub permission_mode: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub write_guard: &'a mut WriteGuard,
    pub plan_active: bool,
    pub turn: usize,
    pub breaker: &'a mut CircuitBreaker,
    pub journal:
        Option<&'a mut crate::services::agent_local::conversation_journal::ConversationJournal>,
    pub tools: &'a mut crate::services::agent_local::extension_tool_set::ExtensionToolSet,
    pub subagents:
        &'a mut crate::services::agent_local::subagent_orchestration::ParentSubagentOrchestrator,
    pub compression: &'a super::agent_loop_compression::LoopCompression<'a>,
    pub last_prompt: &'a mut Option<u32>,
    pub last_eval: &'a mut Option<u32>,
    pub fixture_mode: bool,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<&'a mut crate::services::reasoning_fixture_run::FixtureRunContext>,
}

pub(super) struct ToolTurnOutput {
    pub stop: bool,
    pub previews: crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
}

pub(super) async fn prepare_tool_batch(
    tool_calls: &[(String, serde_json::Value)],
    turn: usize,
    breaker: &mut CircuitBreaker,
) -> Result<bool, String> {
    if turn == MAX_TURNS - 1 {
        return Err(agent_loop_errors::max_turns_message());
    }
    breaker.check(tool_calls)?;
    Ok(crate::services::agent_local::subagent_tool_control::is_control_only(tool_calls))
}

pub(super) struct ToolBatchContext<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub messages: &'a mut Vec<ChatMessage>,
    pub tool_calls: &'a [(String, serde_json::Value)],
    pub tool_call_ids: &'a [String],
    pub working_dir: &'a Path,
    pub permission_mode: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub write_guard: &'a mut WriteGuard,
    pub plan_active: bool,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<&'a mut crate::services::reasoning_fixture_run::FixtureRunContext>,
}

pub(super) async fn execute_tool_batch(context: ToolBatchContext<'_>) -> ToolExecutionOutcome {
    #[cfg(debug_assertions)]
    if let Some(run) = context.fixture_run {
        return crate::services::agent_local::fixture_tool_executor::execute(
            context.on_event,
            context.messages,
            context.tool_calls,
            context.tool_call_ids,
            run,
            &context.cancel,
        )
        .await;
    }
    crate::services::agent_local::tool_executor::run_tools(
        context.on_event,
        context.messages,
        context.tool_calls,
        context.working_dir,
        context.permission_mode,
        context.session_id,
        context.request_id,
        context.cancel,
        context.write_guard,
        context.plan_active,
        context.tool_call_ids,
        None,
    )
    .await
}

pub(super) async fn run_tool_turn(
    mut context: ToolTurnContext<'_>,
) -> Result<ToolTurnOutput, String> {
    let control_only =
        prepare_tool_batch(&context.result.tool_calls, context.turn, context.breaker).await?;
    let tool_start = context.messages.len();
    let mut outcome = execute_tool_batch(ToolBatchContext {
        on_event: context.on_event,
        messages: context.messages,
        tool_calls: &context.result.tool_calls,
        tool_call_ids: &context.result.tool_call_ids,
        working_dir: context.working_dir,
        permission_mode: context.permission_mode,
        session_id: context.session_id,
        request_id: context.request_id,
        cancel: context.cancel.clone(),
        write_guard: context.write_guard,
        plan_active: context.plan_active,
        #[cfg(debug_assertions)]
        fixture_run: context.fixture_run.as_deref_mut(),
    })
    .await;
    let compressed = outcome.compressed;
    let tool_end = context.messages.len();
    let stop = outcome.apply_follow_ups(&mut context.messages[tool_start..tool_end])?;
    if let Some(journal) = context.journal.as_deref_mut() {
        journal
            .persist_tool_results(&context.messages[tool_start..tool_end], outcome.artifacts())
            .await?;
    }
    let previews = outcome.take_artifact_previews().await;
    refresh_tools(
        context.tools,
        context.session_id,
        context.request_id,
        context.fixture_mode,
    )
    .await?;
    context
        .subagents
        .wait_after_tool_batch(control_only, context.messages, context.cancel.clone())
        .await?;
    context
        .compression
        .finish_tools(
            context.messages,
            compressed,
            super::agent_loop_compression::LastCounts::new(context.last_prompt, context.last_eval),
            context.cancel,
        )
        .await;
    Ok(ToolTurnOutput { stop, previews })
}

async fn refresh_tools(
    tools: &mut crate::services::agent_local::extension_tool_set::ExtensionToolSet,
    session_id: &str,
    request_id: &str,
    fixture_mode: bool,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    if fixture_mode {
        return Ok(());
    }
    #[cfg(not(debug_assertions))]
    let _ = fixture_mode;
    crate::services::agent_local::extension_tool_set::refresh_and_record(
        tools, session_id, request_id,
    )
    .await
}
