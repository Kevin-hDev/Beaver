use super::subagent_read_only_command_test_support::{
    child_session, cleanup, snapshot, SUBAGENT_READ_ONLY,
};
use crate::services::agent_local::parent_message_inbox::ParentMessageInbox;
use crate::ActiveStreams;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn chat_admission_rejects_a_child_before_runtime_or_disk_mutation() {
    let session = child_session("Stream child").await;
    let before_session = snapshot(&session.id).await;
    let before_request_starts = request_start_count(&session);
    let sentinel_inbox = Arc::new(ParentMessageInbox::new());
    let sentinel_token = CancellationToken::new();
    let streams = ActiveStreams(Mutex::new(HashMap::from([(
        session.id.clone(),
        (
            sentinel_token.clone(),
            73,
            "sentinel-request".to_string(),
            sentinel_inbox.clone(),
        ),
    )])));
    let request_session_id = session.id.clone();

    let result = super::agent_chat_admission::admit(
        &session.id,
        Some("manual"),
        &streams,
        |_| async {},
        |generation| async move {
            crate::services::agent_local::stream_diagnostics::start_request(
                &request_session_id,
                generation,
            )
            .await
        },
    )
    .await;
    let after_session = snapshot(&session.id).await;
    let after_document = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("child session remains readable");

    {
        let map = streams.0.lock().await;
        assert_eq!(map.len(), 1);
        let (token, generation, request_id, inbox) = map.get(&session.id).expect("sentinel");
        assert!(!token.is_cancelled());
        assert_eq!(*generation, 73);
        assert_eq!(request_id, "sentinel-request");
        assert!(Arc::ptr_eq(inbox, &sentinel_inbox));
    }
    sentinel_token.cancel();
    cleanup(&session).await;

    assert_eq!(
        result.as_ref().err().map(String::as_str),
        Some(SUBAGENT_READ_ONLY)
    );
    assert_eq!(after_session, before_session);
    assert_eq!(request_start_count(&after_document), before_request_starts);
}

fn request_start_count(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> usize {
    session
        .diagnostic_runs
        .iter()
        .flat_map(|run| &run.events)
        .filter(|event| event.phase == "request_start")
        .count()
}
