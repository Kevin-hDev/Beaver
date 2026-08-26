use crate::services::agent_local::parent_message_inbox::ParentMessageInbox;
use crate::ActiveStreams;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[test]
fn all_post_start_failures_use_the_same_rollback_boundary() {
    let run = include_str!("agent_chat_run.rs");
    let spawn = include_str!("agent_chat_run_spawn.rs");
    assert!(run.matches("rollback(streams").count() >= 4);
    assert!(spawn.contains("rollback(streams, &session_id, &stream)"));
}

#[tokio::test]
async fn rollback_terminalizes_the_current_request_exactly_once() {
    let session = session("Current rollback").await;
    let stream = admission(&session.id, 1).await;
    let streams = ActiveStreams(Mutex::new(HashMap::from([(
        session.id.clone(),
        entry(&stream),
    )])));

    super::agent_chat_run::rollback(&streams, &session.id, &stream).await;
    super::agent_chat_run::rollback(&streams, &session.id, &stream).await;

    assert!(streams.0.lock().await.is_empty());
    assert!(stream.cancel.is_cancelled());
    assert!(!stream.parent_message_inbox.enqueue(turn()).await.unwrap());
    assert_terminal(&session.id, "failed", 1).await;
    cleanup(&session.id).await;
}

#[tokio::test]
async fn stale_rollback_preserves_the_replacement_cancellation_terminal() {
    let session = session("Stale rollback").await;
    let old = admission(&session.id, 1).await;
    crate::services::agent_local::stream_diagnostics::record_cancelled(
        &session.id,
        &old.request_id,
    )
    .await;
    let current = admission(&session.id, 2).await;
    let streams = ActiveStreams(Mutex::new(HashMap::from([(
        session.id.clone(),
        entry(&current),
    )])));

    super::agent_chat_run::rollback(&streams, &session.id, &old).await;

    let map = streams.0.lock().await;
    assert_eq!(map.get(&session.id).unwrap().1, 2);
    drop(map);
    assert_terminal(&session.id, "cancelled", 1).await;
    cleanup(&session.id).await;
}

async fn session(title: &str) -> crate::services::agent_local::types_session::AgentSession {
    crate::services::agent_local::session_store::create_full(
        title,
        "qwen3.5:4b",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap()
}

async fn admission(
    session_id: &str,
    generation: u64,
) -> super::agent_chat_admission::AgentChatAdmission {
    super::agent_chat_admission::AgentChatAdmission {
        cancel: CancellationToken::new(),
        generation,
        parent_message_inbox: Arc::new(ParentMessageInbox::new()),
        permission_mode: "manual".into(),
        request_id: crate::services::agent_local::stream_diagnostics::start_request(
            session_id, generation,
        )
        .await,
    }
}

fn entry(
    stream: &super::agent_chat_admission::AgentChatAdmission,
) -> super::agent_chat_streams::StreamEntry {
    (
        stream.cancel.clone(),
        stream.generation,
        stream.request_id.clone(),
        Arc::clone(&stream.parent_message_inbox),
    )
}

fn turn() -> crate::models::agent_turn_contract::NewUserTurnInput {
    crate::models::agent_turn_contract::NewUserTurnInput {
        content: "late".into(),
        files: Vec::new(),
        skills: Vec::new(),
    }
}

async fn assert_terminal(session_id: &str, status: &str, failed_events: usize) {
    let stored = crate::services::agent_local::session_store::get(session_id)
        .await
        .unwrap();
    let run = stored.diagnostic_runs.first().unwrap();
    assert_eq!(run.status, status);
    assert!(run.ended_at.is_some());
    assert_eq!(
        run.events
            .iter()
            .filter(|event| event.phase == "failed")
            .count(),
        failed_events
    );
}

async fn cleanup(session_id: &str) {
    crate::services::agent_local::session_store::delete_one(session_id)
        .await
        .unwrap();
}
