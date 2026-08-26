use crate::models::agent_turn_contract::{NewUserTurnInput, TurnStart};
use crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate;
use crate::services::agent_local::parent_message_inbox::ParentMessageInbox;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};
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

#[tokio::test]
async fn projectless_empty_fixture_chat_admits_before_workspace_resolution() {
    let session = session("Fixture projectless admission").await;
    let stream = admission(&session.id, 7).await;
    let streams = ActiveStreams(Mutex::new(HashMap::from([(
        session.id.clone(),
        entry(&stream),
    )])));

    let admitted = super::agent_chat_turn::admit_current(
        &streams,
        &session.id,
        stream.generation,
        prepared_turn("fixture prompt").await,
        forbidden_target(),
        reasoning_update(&session),
    )
    .await
    .expect("the durable fixture turn is admitted first");
    let workspace = super::agent_working_dir::resolve_for_session(&session.id, None)
        .await
        .expect("a projectless admitted fixture gets its managed workspace");
    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("fixture session remains readable");

    assert!(workspace.outputs_dir.is_some());
    assert!(stored.working_dir_managed);
    assert_eq!(stored.messages.len(), 1);
    assert_eq!(stored.messages[0].id, admitted.turn.user_message_id);
    assert_eq!(stored.messages[0].content, "fixture prompt");
    cleanup(&session.id).await;
}

#[tokio::test]
async fn projectless_main_chat_rolls_back_the_durable_turn_when_workspace_resolution_fails() {
    let session = session("Main chat working directory rollback").await;
    let stream = admission(&session.id, 8).await;
    let streams = ActiveStreams(Mutex::new(HashMap::from([(
        session.id.clone(),
        entry(&stream),
    )])));
    let admitted = super::agent_chat_turn::admit_current(
        &streams,
        &session.id,
        stream.generation,
        prepared_turn("must not become orphaned").await,
        forbidden_target(),
        reasoning_update(&session),
    )
    .await
    .expect("the durable main-chat turn is admitted first");

    let missing = format!("/private/beaver-fixture-missing-{}", uuid::Uuid::new_v4());
    assert!(
        super::agent_working_dir::resolve_for_session(&session.id, Some(&missing))
            .await
            .is_err()
    );
    super::agent_chat_turn::rollback_current(
        &streams,
        &session.id,
        stream.generation,
        &admitted.rollback(),
    )
    .await
    .expect("failed working-directory setup rolls back the admitted turn");

    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("session remains readable");
    assert!(stored.messages.is_empty(), "no orphaned prompt remains");
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

async fn prepared_turn(content: &str) -> super::agent_chat_turn::PreparedTurn {
    super::agent_chat_turn::prepare(TurnStart::New(NewUserTurnInput {
        content: content.to_string(),
        files: Vec::new(),
        skills: Vec::new(),
    }))
    .await
    .expect("prepare input")
}

fn forbidden_target() -> ContinuationTarget {
    ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".to_string(),
        reasoning_mode: ReasoningModeId::Off,
    })
}

fn reasoning_update(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> SessionReasoningUpdate {
    let profile = crate::services::reasoning_profile::EffectiveReasoningProfile::api(
        "fixture", "fixture", None, false, false,
    )
    .expect("off profile");
    SessionReasoningUpdate::new(session, &profile)
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
