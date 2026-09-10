pub(crate) async fn admit_automation_current(
    streams: &crate::ActiveStreams,
    session_id: &str,
    generation: u64,
    turn: super::PreparedTurn,
    target: crate::services::reasoning_continuity::contract::ContinuationTarget,
    reasoning: crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate,
) -> Result<super::AdmittedCurrentTurn, String> {
    super::admit_current_with_kind(
        streams, session_id, generation, turn, target, reasoning, true,
    )
    .await
}
