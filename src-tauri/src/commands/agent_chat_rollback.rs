use crate::ActiveStreams;

pub(crate) async fn rollback(
    streams: &ActiveStreams,
    session_id: &str,
    stream: &super::agent_chat_admission::AgentChatAdmission,
    failure_code: &str,
) {
    stream.cancel.cancel();
    stream.parent_message_inbox.close().await;
    let mut map = streams.0.lock().await;
    let current = matches!(map.get(session_id), Some((_, generation, _, _)) if *generation == stream.generation);
    if current {
        map.remove(session_id);
    }
    drop(map);
    if current {
        crate::services::agent_local::stream_diagnostics::record_failure(
            session_id,
            Some(&stream.request_id),
            failure_code,
            false,
        )
        .await;
    }
}
