pub(super) async fn rollback(
    streams: &crate::ActiveStreams,
    session_id: &str,
    generation: u64,
    admitted: &crate::commands::agent_chat_turn::AdmissionRollback,
) {
    let _ = crate::commands::agent_chat_turn::rollback_current(
        streams, session_id, generation, admitted,
    )
    .await;
    crate::commands::agent_chat_streams::finish_active_stream(streams, session_id, generation)
        .await;
}
