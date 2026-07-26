use super::{session_store, subagent_orchestration::ParentSubagentOrchestrator};
use super::subagent_terminal_wait_test_support::lock;
use serde_json::json;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[tokio::test(start_paused = true)]
async fn parent_waiter_does_not_wake_without_a_terminal_event() {
    let _guard = lock().await;
    let parent = session_store::create_full(
        "Parent event wait",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create parent");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_id = uuid::Uuid::new_v4().to_string();
    super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let cancel = CancellationToken::new();
    let wait_cancel = cancel.clone();
    let waiter = tokio::spawn(async move {
        orchestrator
            .after_no_tool_turn(&mut Vec::new(), wait_cancel)
            .await
    });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(60 * 60)).await;
    tokio::task::yield_now().await;
    let stayed_suspended = !waiter.is_finished();
    cancel.cancel();
    let outcome = waiter.await.expect("join waiter");

    assert_eq!(outcome, Err("Annulé".to_string()));
    assert!(stayed_suspended, "the parent waiter woke during one hour");
    super::subagent_registry::unregister(&child_id).await;
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn replacement_stream_adopts_an_existing_active_child() {
    let _guard = lock().await;
    let parent = session_store::create_full(
        "Replacement event wait",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create parent");
    let child_id = uuid::Uuid::new_v4().to_string();
    super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child before replacement");
    let mut replacement = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();

    let outcome = tokio::time::timeout(
        Duration::from_millis(25),
        replacement.after_no_tool_turn(&mut messages, CancellationToken::new()),
    )
    .await;

    let stayed_suspended = outcome.is_err();
    super::subagent_registry::unregister(&child_id).await;
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
    assert!(
        stayed_suspended,
        "the replacement stream ignored the existing active child"
    );
}

#[tokio::test]
async fn control_only_batch_waits_but_mixed_batch_continues() {
    let _guard = lock().await;
    let parent = session_store::create_full(
        "Parent control wait",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create parent");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_id = uuid::Uuid::new_v4().to_string();
    super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let mut messages = Vec::new();
    let controls = [("list_subagents".to_string(), json!({}))];

    let control_outcome = tokio::time::timeout(
        Duration::from_millis(25),
        orchestrator.wait_after_tool_batch(
            super::subagent_tool_control::is_control_only(&controls),
            &mut messages,
            CancellationToken::new(),
        ),
    )
    .await;
    let mixed = [
        ("list_subagents".to_string(), json!({})),
        ("read_file".to_string(), json!({})),
    ];
    let mixed_outcome = tokio::time::timeout(
        Duration::from_millis(25),
        orchestrator.wait_after_tool_batch(
            super::subagent_tool_control::is_control_only(&mixed),
            &mut messages,
            CancellationToken::new(),
        ),
    )
    .await;

    super::subagent_registry::unregister(&child_id).await;
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
    assert!(
        control_outcome.is_err(),
        "a control-only batch requested another model turn"
    );
    assert!(matches!(mixed_outcome, Ok(Ok(()))));
}

#[test]
fn orchestrator_contains_no_temporal_wakeup() {
    let source = include_str!("subagent_orchestration.rs");

    for forbidden in ["tokio::time::sleep", "POLL_INTERVAL", "REMINDER_INTERVAL"] {
        assert!(!source.contains(forbidden), "temporal wakeup: {forbidden}");
    }
}

#[test]
fn replacement_cancels_only_parent_while_stop_keeps_owned_child_cancellation() {
    let replacement = include_str!("../../commands/agent_chat.rs");
    let cancel_command = include_str!("../../commands/agent_chat_cancel.rs");

    assert!(replacement.contains("cancel_with_lock"));
    assert!(!replacement.contains("cancel_stopped_parent_stream_children"));
    assert!(cancel_command.contains("cancel_stopped_parent_stream_children"));
}
