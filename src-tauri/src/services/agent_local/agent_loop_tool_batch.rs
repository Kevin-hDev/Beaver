use super::stream_events::AgentEventEmitter;
use super::tool_execution_outcome::ToolExecutionOutcome;
use super::types_ollama::ChatMessage;
use super::types_tools::ToolResult;
use super::write_guard::WriteGuard;
use super::{agent_loop_thinking_retry::EagerHandle, circuit_breaker::CircuitBreaker};
use std::collections::HashMap;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) struct PreparedToolBatch {
    pub control_only: bool,
    pub eager_results: HashMap<usize, ToolResult>,
}

pub(super) async fn prepare(
    eager_handle: EagerHandle,
    fixture_mode: bool,
    tool_calls: &[(String, serde_json::Value)],
    turn: usize,
    model: &str,
    breaker: &mut CircuitBreaker,
) -> Result<PreparedToolBatch, String> {
    super::agent_loop_support::ensure_more_turns(turn, model).await?;
    if let Err(message) = breaker.check(tool_calls) {
        eager_handle.abort();
        super::agent_loop_support::decharge_gpu(model).await;
        return Err(message);
    }
    let control_only = super::subagent_tool_control::is_control_only(tool_calls);
    let eager_results = if fixture_mode {
        eager_handle.abort();
        HashMap::new()
    } else {
        eager_handle.await.unwrap_or_default()
    };
    Ok(PreparedToolBatch {
        control_only,
        eager_results,
    })
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
    pub eager_results: HashMap<usize, ToolResult>,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<&'a mut crate::services::reasoning_fixture_run::FixtureRunContext>,
}

pub(super) async fn execute(context: ToolBatchContext<'_>) -> ToolExecutionOutcome {
    #[cfg(debug_assertions)]
    if let Some(run) = context.fixture_run {
        return super::fixture_tool_executor::execute(
            context.on_event,
            context.messages,
            context.tool_calls,
            context.tool_call_ids,
            run,
            &context.cancel,
        )
        .await;
    }
    super::tool_executor::run_tools_with_eager(
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
        Some(context.eager_results),
        context.tool_call_ids,
        None,
    )
    .await
}
