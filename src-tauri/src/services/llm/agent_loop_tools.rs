use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::circuit_breaker::CircuitBreaker;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_execution_outcome::ToolExecutionOutcome;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::write_guard::WriteGuard;
use std::path::Path;
use tokio_util::sync::CancellationToken;

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
