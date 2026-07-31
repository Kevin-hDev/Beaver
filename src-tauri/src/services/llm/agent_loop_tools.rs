use crate::services::agent_local::agent_loop_errors;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::circuit_breaker::CircuitBreaker;

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
