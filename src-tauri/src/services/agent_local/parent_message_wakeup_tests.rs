use super::parent_message_inbox::ParentMessageInbox;
use super::session_store;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::ChatMessage;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn queued_user_message_wakes_the_parent_without_ending_the_child() {
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
    let waiter = tokio::spawn(async move {
        let mut messages = Vec::new();
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (outcome, messages)
    });

    tokio::task::yield_now().await;
    inbox.enqueue(vec![user("Nouvelle précision")]).await.unwrap();
    let (outcome, messages) = tokio::time::timeout(Duration::from_secs(1), waiter)
        .await
        .expect("waiter timeout")
        .expect("join waiter");

    assert_eq!(outcome, Ok(true));
    assert_eq!(messages.last().map(|message| message.content.as_str()), Some("Nouvelle précision"));
    assert_eq!(super::subagent_registry::active_children_for_parent(&parent.id).await, vec![child_id.clone()]);
    super::subagent_registry::unregister(&child_id).await;
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

fn user(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".into(), content: content.into(), images: None,
        tool_calls: None, tool_name: None, tool_call_id: None, reasoning_content: None,
    }
}
