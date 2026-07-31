use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::circuit_breaker::CircuitBreaker;
use std::path::Path;

pub(super) async fn prepare_tool_batch(
    _session_id: &str,
    _request_id: &str,
    tool_calls: &[(String, serde_json::Value)],
    _working_dir: &Path,
    turn: usize,
    breaker: &mut CircuitBreaker,
) -> Result<bool, String> {
    if turn == MAX_TURNS - 1 {
        return Err(agent_loop_errors::max_turns_message());
    }
    breaker.check(tool_calls)?;
    Ok(crate::services::agent_local::subagent_tool_control::is_control_only(tool_calls))
}
