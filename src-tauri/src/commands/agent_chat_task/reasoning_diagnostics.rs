pub(super) async fn record_persisted(
    session_id: &str,
    request_id: &str,
    turn_id: &str,
    assistant_message_id: &str,
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
