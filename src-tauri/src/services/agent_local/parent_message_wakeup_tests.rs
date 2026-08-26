use super::parent_message_inbox::ParentMessageInbox;
use super::session_store;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use crate::models::agent_turn_contract::NewUserTurnInput;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn queued_user_message_waits_for_the_future_durable_commit_boundary() {
    let _guard = super::subagent_terminal_wait_test_support::lock().await;
    let parent = session_store::create_full("Parent input", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let child_id = uuid::Uuid::new_v4().to_string();
    super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let inbox = Arc::new(ParentMessageInbox::new());
    let mut orchestrator =
        ParentSubagentOrchestrator::with_parent_inbox(&parent.id, Some(inbox.clone())).await;
    let cancel = CancellationToken::new();
    let waiter_cancel = cancel.clone();
    let waiter = tokio::spawn(async move {
        let mut messages = Vec::new();
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, waiter_cancel)
            .await;
        (outcome, messages)
    });

    tokio::task::yield_now().await;
    inbox.enqueue(user("Nouvelle précision")).await.unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(!waiter.is_finished());
    assert_eq!(inbox.len().await, 1);
    cancel.cancel();
    let (outcome, messages) = waiter.await.expect("join waiter");

    assert_eq!(outcome, Err("Annulé".to_string()));
    assert!(messages.is_empty());
    assert_eq!(super::subagent_registry::active_children_for_parent(&parent.id).await, vec![child_id.clone()]);
    super::subagent_registry::unregister(&child_id).await;
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

fn user(content: &str) -> NewUserTurnInput {
    NewUserTurnInput { content: content.into(), files: Vec::new(), skills: Vec::new() }
}
