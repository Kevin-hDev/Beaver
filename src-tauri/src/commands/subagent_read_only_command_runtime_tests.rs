use super::subagent_read_only_command_test_support::{
    child_session, cleanup, snapshot, SUBAGENT_READ_ONLY,
};
use crate::models::agent_turn_contract::NewUserTurnInput;
use crate::services::agent_local::parent_message_inbox::ParentMessageInbox;
use crate::ActiveStreams;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn queue_agent_message_rejects_a_child_without_changing_the_active_inbox() {
    let session = child_session("Queued child message").await;
    let before_session = snapshot(&session.id).await;
    let sentinel_inbox = Arc::new(ParentMessageInbox::new());
    let sentinel_token = CancellationToken::new();
    let app = tauri::test::mock_builder()
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("build isolated Tauri app");

    {
        let streams = app.state::<ActiveStreams>();
        streams.0.lock().await.insert(
            session.id.clone(),
            (
                sentinel_token.clone(),
                41,
                "sentinel-request".to_string(),
                sentinel_inbox.clone(),
            ),
        );
    }

    let result = super::agent_chat_queue::queue_agent_message(
        session.id.clone(),
        41,
        turn("must never be queued"),
        app.state::<ActiveStreams>(),
    )
    .await;
    let after_session = snapshot(&session.id).await;
    let queued_intentions = sentinel_inbox.len().await;

    {
        let streams = app.state::<ActiveStreams>();
        let map = streams.0.lock().await;
        assert_eq!(map.len(), 1);
        let (token, generation, request_id, inbox) = map.get(&session.id).expect("sentinel");
        assert!(!token.is_cancelled());
        assert_eq!(*generation, 41);
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
    assert_eq!(queued_intentions, 0);
}

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

#[tokio::test]
async fn agent_chat_queue_rejects_a_stale_generation_without_mutation() {
    let session = crate::services::agent_local::session_store::create_full(
        "Stale queue",
        "qwen3.5:4b",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    let inbox = Arc::new(ParentMessageInbox::new());
    let app = tauri::test::mock_builder()
        .manage(ActiveStreams(Mutex::new(HashMap::from([(
            session.id.clone(),
            (CancellationToken::new(), 42, "active".into(), inbox.clone()),
        )]))))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("build isolated Tauri app");

    let result = super::agent_chat_queue::queue_agent_message(
        session.id.clone(),
        41,
        turn("stale"),
        app.state::<ActiveStreams>(),
    )
    .await;

    assert_eq!(result, Ok(false));
    assert_eq!(inbox.len().await, 0);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn malformed_queue_session_id_keeps_the_public_generic_error() {
    let app = tauri::test::mock_builder()
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("build isolated Tauri app");

    let result = super::agent_chat_queue::queue_agent_message(
        "../invalid".to_string(),
        1,
        turn("invalid"),
        app.state::<ActiveStreams>(),
    )
    .await;

    assert_eq!(result, Err("Impossible d'envoyer ce message".to_string()));
}

fn turn(content: &str) -> NewUserTurnInput {
    NewUserTurnInput {
        content: content.to_string(),
        files: Vec::new(),
        skills: Vec::new(),
    }
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
