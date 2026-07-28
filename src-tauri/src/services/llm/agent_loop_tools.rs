use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::circuit_breaker::CircuitBreaker;
use std::path::Path;

pub(super) async fn record_detected_tool_calls(
    session_id: &str,
    request_id: &str,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &Path,
) {
    for (name, args) in tool_calls {
        crate::services::agent_local::tool_executor_diagnostics::detected(
            session_id,
            request_id,
            name,
            args,
            working_dir,
        )
        .await;
    }
}

pub(super) async fn prepare_tool_batch(
    session_id: &str,
    request_id: &str,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &Path,
    turn: usize,
    breaker: &mut CircuitBreaker,
) -> Result<bool, String> {
    record_detected_tool_calls(session_id, request_id, tool_calls, working_dir).await;
    if turn == MAX_TURNS - 1 {
        return Err(agent_loop_errors::max_turns_message());
    }
    breaker.check(tool_calls)?;
    Ok(crate::services::agent_local::subagent_tool_control::is_control_only(tool_calls))
}
