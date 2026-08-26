pub(super) async fn record_persisted(
    session_id: &str,
    request_id: &str,
    turn_id: &str,
    assistant_message_id: &str,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) {
    let Ok(session) = crate::services::agent_local::session_store::get(session_id).await else {
        return;
    };
    let Some(envelope) = session
        .messages
        .iter()
        .find(|message| message.id == assistant_message_id)
        .and_then(|message| message.continuation.as_ref())
    else {
        return;
    };
    use crate::services::reasoning_continuity::diagnostics::{record_envelope, ReasoningDecision};
    record_replayed(
        session_id,
        request_id,
        assistant_message_id,
        &session,
        continuation_target,
    )
    .await;
    record_envelope(
        session_id,
        request_id,
        turn_id,
        ReasoningDecision::Captured,
        envelope,
    )
    .await;
    record_envelope(
        session_id,
        request_id,
        turn_id,
        ReasoningDecision::Persisted,
        envelope,
    )
    .await;
}

async fn record_replayed(
    session_id: &str,
    request_id: &str,
    current_assistant_id: &str,
    session: &crate::services::agent_local::types_session::AgentSession,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) {
    use crate::services::reasoning_continuity::contract::ContinuationUse;
    use crate::services::reasoning_continuity::diagnostics::{record_envelope, ReasoningDecision};

    let Some(current_index) = session
        .messages
        .iter()
        .position(|message| message.id == current_assistant_id)
    else {
        return;
    };
    let Some(base_target) = continuation_target.and_then(|target| target.replay()) else {
        return;
    };
    for (index, message) in session.messages[..current_index].iter().enumerate() {
        let Some(envelope) = message.continuation.as_ref() else {
            continue;
        };
        let continuation_use = if session
            .messages
            .get(index + 1)
            .is_some_and(|next| next.turn_id == message.turn_id && next.role == "tool")
        {
            ContinuationUse::ToolContinuation
        } else {
            ContinuationUse::UserContinuation
        };
        let mut target = base_target.clone();
        target.continuation_use = continuation_use;
        #[cfg(debug_assertions)]
        let decision = if continuation_target.is_some_and(|target| target.is_fixture_candidate()) {
            crate::services::reasoning_continuity::eligibility::decide_fixture_candidate(
                envelope, &target,
            )
        } else {
            crate::services::reasoning_continuity::eligibility::decide(envelope, &target)
        };
        #[cfg(not(debug_assertions))]
        let decision =
            { crate::services::reasoning_continuity::eligibility::decide(envelope, &target) };
        if decision != crate::services::reasoning_continuity::eligibility::ReplayDecision::Allowed {
            continue;
        }
        record_envelope(
            session_id,
            request_id,
            &message.turn_id,
            ReasoningDecision::Replayed,
            envelope,
        )
        .await;
    }
}
