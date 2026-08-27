use super::agent_chat_streams::{
    replace_active_stream, ACTIVE_STREAM_LIMIT_REACHED, STREAM_REPLACED,
};
use crate::services::agent_local::subagent_registry;
use crate::ActiveStreams;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

#[test]
fn chat_stream_uses_the_tested_replacement_path() {
    let command = include_str!("agent_chat_run.rs");
    let admission = include_str!("agent_chat_admission.rs");

    assert!(
        command.contains("agent_chat_admission::admit"),
        "chat_stream contourne la frontière d'admission testable"
    );
    assert!(
        admission.contains("agent_chat_streams::replace_active_stream"),
        "l'admission contourne la frontière de remplacement testable"
    );
}

#[test]
fn replacement_finishes_before_the_new_work_admission() {
    let source = include_str!("agent_chat_run.rs");
    let replacement = source
        .find("let stream = admit_stream")
        .expect("stream replacement admission boundary");
    let admission = source
        .find("agent_chat_work::admit")
        .expect("stream admission");

    assert!(
        replacement < admission,
        "a replacement temporarily consumes two stream admissions"
    );
}

#[test]
fn capacity_precedes_resolution_and_durable_admission_precedes_spawn() {
    let source = include_str!("agent_chat_run.rs");
    let capacity = source.find("agent_chat_work::admit").unwrap();
    let target = source.find("agent_chat_target::resolve").unwrap();
    let resolution = source.find("agent_chat_turn::prepare").unwrap();
    let durable = source.find("agent_chat_turn::admit").unwrap();
    let working_dir = source
        .find("agent_working_dir::resolve_for_session")
        .unwrap();
    let spawn = source.find("spawn(").unwrap();

    assert!(capacity < target);
    assert!(target < resolution);
    assert!(resolution < durable);
    assert!(
        durable < working_dir,
        "le répertoire de travail ne peut pas précéder l'admission durable"
    );
    assert!(
        working_dir < spawn,
        "provider work can start before the user is durable"
    );
}

#[test]
fn active_user_message_is_refused_until_a_durable_consumer_exists() {
    let backend = include_str!("agent_chat_queue.rs");

    assert!(backend.contains("pub async fn queue_agent_message"));
    assert!(!backend.contains("inbox.enqueue"));
    assert!(backend.contains("Ok(false)"));
}

#[tokio::test]
async fn later_start_wins_while_previous_cancellation_is_suspended() {
    let streams = Arc::new(ActiveStreams(Mutex::new(HashMap::new())));
    let session_id = id();
    let child_id = id();
    let old_owner = CancellationToken::new();
    let child_cancel = CancellationToken::new();
    streams.0.lock().await.insert(
        session_id.clone(),
        (old_owner.clone(), 0, "request-old".to_string(), inbox()),
    );
    subagent_registry::register_execution_for_parent_stream(
        &session_id,
        &child_id,
        child_cancel.clone(),
        None,
        &old_owner,
    )
    .await
    .expect("register child");

    let first_cancel = CancellationToken::new();
    let second_cancel = CancellationToken::new();
    let (at_boundary_tx, at_boundary_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let first_task = {
        let streams = streams.clone();
        let session_id = session_id.clone();
        let first_cancel = first_cancel.clone();
        tokio::spawn(async move {
            replace_active_stream(
                &streams,
                &session_id,
                first_cancel,
                1,
                inbox(),
                move |(old_cancel, _, _, _)| async move {
                    let _ = at_boundary_tx.send(());
                    let _ = release_rx.await;
                    old_cancel.cancel();
                },
                || async { "request-a".to_string() },
            )
            .await
        })
    };
    at_boundary_rx
        .await
        .expect("first start reached cancellation");

    replace_active_stream(
        &streams,
        &session_id,
        second_cancel.clone(),
        2,
        inbox(),
        |(old_cancel, _, _, _)| async move { old_cancel.cancel() },
        || async { "request-b".to_string() },
    )
    .await
    .expect("second start");
    release_tx.send(()).expect("release first start");
    let first_result = first_task.await.expect("join first start");

    let (tracked_count, tracked_generation, tracked_request) = {
        let map = streams.0.lock().await;
        let (_, generation, request_id, _) = map.get(&session_id).expect("tracked winner");
        (map.len(), *generation, request_id.clone())
    };
    second_cancel.cancel();
    subagent_registry::cancel_stopped_parent_stream_children(&session_id).await;
    let winner_owns_child = child_cancel.is_cancelled();
    subagent_registry::unregister(&child_id).await;

    assert_eq!(first_result, Err(STREAM_REPLACED.to_string()));
    assert_eq!(tracked_count, 1);
    assert_eq!(tracked_generation, 2);
    assert_eq!(tracked_request, "request-b");
    assert!(first_cancel.is_cancelled(), "le writer perdant reste actif");
    assert!(
        winner_owns_child,
        "l'enfant appartient encore au writer perdant"
    );
}

#[tokio::test]
async fn a_new_session_is_rejected_with_the_stable_capacity_code() {
    let streams = ActiveStreams(Mutex::new(HashMap::new()));
    let mut map = streams.0.lock().await;
    for generation in
        0..crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
    {
        map.insert(
            id(),
            (CancellationToken::new(), generation as u64, id(), inbox()),
        );
    }
    drop(map);

    let result = replace_active_stream(
        &streams,
        &id(),
        CancellationToken::new(),
        0,
        inbox(),
        |_| async {},
        || async { id() },
    )
    .await;

    assert_eq!(result, Err(ACTIVE_STREAM_LIMIT_REACHED.to_string()));
}

#[tokio::test]
async fn a_request_started_before_a_capacity_race_is_terminalized_once() {
    let streams = Arc::new(ActiveStreams(Mutex::new(HashMap::new())));
    let session = crate::services::agent_local::session_store::create_full(
        "Capacity terminal",
        "qwen3.5:4b",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    let start_streams = Arc::clone(&streams);
    let start_session = session.id.clone();

    let result = replace_active_stream(
        &streams,
        &session.id,
        CancellationToken::new(),
        1,
        inbox(),
        |_| async {},
        move || async move {
            let request_id =
                crate::services::agent_local::stream_diagnostics::start_request(&start_session, 1)
                    .await;
            let mut map = start_streams.0.lock().await;
            for generation in
                0..crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
            {
                map.insert(
                    id(),
                    (CancellationToken::new(), generation as u64, id(), inbox()),
                );
            }
            request_id
        },
    )
    .await;

    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    let run = stored.diagnostic_runs.last().unwrap();
    assert_eq!(result, Err(ACTIVE_STREAM_LIMIT_REACHED.to_string()));
    assert_eq!(run.status, "failed");
    assert!(run.ended_at.is_some());
    assert_eq!(
        run.events
            .iter()
            .filter(|event| event.phase == "failed")
            .count(),
        1
    );
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn replacement_before_admission_linearization_never_persists_the_old_user() {
    let streams = Arc::new(ActiveStreams(Mutex::new(HashMap::new())));
    let session = crate::services::agent_local::session_store::create_full(
        "Concurrent admission",
        "qwen3.5:4b",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    replace_active_stream(
        &streams,
        &session.id,
        CancellationToken::new(),
        1,
        inbox(),
        |_| async {},
        || async { "request-a".into() },
    )
    .await
    .unwrap();

    let lease = crate::services::agent_local::session_store::lock_session(&session.id).await;
    let guard = lease.lock().await;
    let (ready_tx, ready_rx) = oneshot::channel();
    let replacing = {
        let streams = Arc::clone(&streams);
        let session_id = session.id.clone();
        tokio::spawn(async move {
            replace_active_stream(
                &streams,
                &session_id,
                CancellationToken::new(),
                2,
                inbox(),
                |_| async {},
                || async {
                    let _ = ready_tx.send(());
                    "request-b".into()
                },
            )
            .await
        })
    };
    ready_rx.await.unwrap();
    let admitting = {
        let streams = Arc::clone(&streams);
        let session_id = session.id.clone();
        tokio::spawn(async move {
            let input = crate::services::agent_local::conversation_input::resolve_with_key(
                crate::models::agent_turn_contract::NewUserTurnInput {
                    content: "stale".into(),
                    files: Vec::new(),
                    skills: Vec::new(),
                },
                &[],
            )
            .await
            .unwrap();
            let stored = crate::services::agent_local::session_store::get(&session_id)
                .await
                .unwrap();
            let reasoning = crate::services::reasoning_profile::EffectiveReasoningProfile::ollama(
                "qwen3.5:4b",
                Some("off"),
                false,
                Some(&["thinking".into()]),
            )
            .unwrap();
            let reasoning = crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate::new(
                &stored,
                &reasoning,
            );
            super::agent_chat_turn::admit_current(
                &streams,
                &session_id,
                1,
                super::agent_chat_turn::PreparedTurn::New(input),
                crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(
                    crate::services::reasoning_continuity::contract::ReplayTarget {
                        route_id: crate::services::reasoning_continuity::contract::RouteId::Ollama,
                        model_id: "qwen3.5:4b".into(),
                        credential_scope: crate::services::reasoning_continuity::contract::CredentialScope::local_uncredentialed(),
                        reasoning_mode: crate::services::reasoning_continuity::contract::ReasoningModeId::Off,
                        continuation_use: crate::services::reasoning_continuity::contract::ContinuationUse::UserContinuation,
                    },
                ),
                reasoning,
            )
            .await
        })
    };
    drop(guard);

    replacing.await.unwrap().unwrap();
    assert!(admitting.await.unwrap().is_err());
    assert!(
        crate::services::agent_local::session_store::get(&session.id)
            .await
            .unwrap()
            .messages
            .is_empty()
    );
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn inbox() -> Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox> {
    Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new())
}
