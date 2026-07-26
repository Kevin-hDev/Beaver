use super::parent_message_inbox::ParentMessageInbox;
use super::types_ollama::ChatMessage;

#[tokio::test]
async fn queued_messages_are_drained_in_order() {
    let inbox = ParentMessageInbox::new();
    inbox.enqueue(vec![user("premier")]).await.unwrap();
    inbox.enqueue(vec![user("second")]).await.unwrap();

    let mut messages = Vec::new();
    assert_eq!(inbox.drain_into(&mut messages).await, 2);
    assert_eq!(messages[0].content, "premier");
    assert_eq!(messages[1].content, "second");
}

#[tokio::test]
async fn finish_is_atomic_with_the_last_message() {
    let inbox = ParentMessageInbox::new();
    inbox.enqueue(vec![user("suite")]).await.unwrap();

    let mut messages = Vec::new();
    assert!(inbox.finish_or_drain(&mut messages).await);
    assert_eq!(messages.len(), 1);
    assert!(inbox.enqueue(vec![user("trop tard")]).await.unwrap());

    assert!(inbox.finish_or_drain(&mut messages).await);
    assert_eq!(messages.len(), 2);
    assert!(!inbox.finish_or_drain(&mut messages).await);
    assert!(!inbox.enqueue(vec![user("fermé")]).await.unwrap());
}

#[tokio::test]
async fn inbox_rejects_more_than_eight_waiting_batches() {
    let inbox = ParentMessageInbox::new();
    for index in 0..8 {
        inbox.enqueue(vec![user(&index.to_string())]).await.unwrap();
    }
    assert!(inbox.enqueue(vec![user("neuf")]).await.is_err());
}

fn user(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".into(),
        content: content.into(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}
