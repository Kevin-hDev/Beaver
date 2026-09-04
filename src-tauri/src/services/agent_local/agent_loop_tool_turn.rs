use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::agent_loop_thinking_retry::EagerHandle;
use super::circuit_breaker::CircuitBreaker;
use super::conversation_journal::ConversationJournal;
use super::extension_tool_set::ExtensionToolSet;
use super::stream_events::AgentEventEmitter;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::{ChatMessage, StreamResult};
use super::write_guard::WriteGuard;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct ToolTurnContext<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub messages: &'a mut Vec<ChatMessage>,
    pub eager_handle: Option<EagerHandle>,
    pub result: &'a StreamResult,
    pub working_dir: &'a Path,
    pub permission_mode: &'a str,
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub cancel: CancellationToken,
    pub write_guard: &'a mut WriteGuard,
    pub plan_active: bool,
    pub fixture_mode: bool,
    pub turn: usize,
    pub model: &'a str,
    pub breaker: &'a mut CircuitBreaker,
    pub journal: Option<&'a mut ConversationJournal>,
    pub tools: &'a mut ExtensionToolSet,
    pub subagents: &'a mut ParentSubagentOrchestrator,
    pub compression: &'a LoopCompression<'a>,
    pub last_prompt: &'a mut Option<u32>,
    pub last_eval: &'a mut Option<u32>,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<&'a mut crate::services::reasoning_fixture_run::FixtureRunContext>,
}

pub(super) async fn run(mut context: ToolTurnContext<'_>) -> Result<bool, String> {
    let eager_handle = context
        .eager_handle
        .take()
        .ok_or_else(|| "tool_execution_failed".to_string())?;
    let prepared = super::agent_loop_tool_batch::prepare(
        eager_handle,
        context.fixture_mode,
        &context.result.tool_calls,
        context.turn,
        context.model,
        context.breaker,
    )
    .await?;
    finish_prepared(context, prepared).await
}

async fn finish_prepared(
    mut context: ToolTurnContext<'_>,
    prepared: super::agent_loop_tool_batch::PreparedToolBatch,
) -> Result<bool, String> {
    let tool_start = context.messages.len();
    let mut outcome =
        super::agent_loop_tool_batch::execute(super::agent_loop_tool_batch::ToolBatchContext {
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
            eager_results: prepared.eager_results,
            #[cfg(debug_assertions)]
            fixture_run: context.fixture_run.as_deref_mut(),
        })
        .await;
    let compressed = outcome.compressed;
    let tool_end = context.messages.len();
    let stop = outcome.apply_follow_ups(&mut context.messages[tool_start..tool_end])?;
    if let Some(journal) = context.journal.as_deref_mut() {
        journal
            .persist_tool_results(&context.messages[tool_start..tool_end])
            .await?;
    }
    refresh_tools(
        context.tools,
        context.session_id,
        context.request_id,
        context.fixture_mode,
    )
    .await?;
    context
        .subagents
        .wait_after_tool_batch(
            prepared.control_only,
            context.messages,
            context.cancel.clone(),
        )
        .await?;
    context
        .compression
        .finish_tools(
            context.messages,
            compressed,
            LastCounts::new(context.last_prompt, context.last_eval),
            context.cancel,
        )
        .await;
    Ok(stop)
}

async fn refresh_tools(
    tools: &mut ExtensionToolSet,
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
    super::extension_tool_set::refresh_and_record(tools, session_id, request_id).await
}
