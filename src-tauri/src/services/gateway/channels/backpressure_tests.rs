use super::backpressure::{try_enqueue, EnqueueOutcome};
use super::InboundMessage;
use crate::services::gateway::types::ChannelKey;

fn message(id: &str) -> InboundMessage {
    InboundMessage {
        channel_key: ChannelKey::new("discord", "main"),
        user_id: "user".into(),
        content: "hello".into(),
        message_id: id.into(),
        chat_id: "chat".into(),
        thread_id: None,
        is_group: false,
        mentions_bot: false,
    }
}

#[tokio::test]
async fn full_queue_never_suspends_the_channel_loop() {
    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    sender.try_send(message("first")).unwrap();

    let outcome = tokio::time::timeout(std::time::Duration::from_millis(20), async {
        try_enqueue(
            &sender,
            message("second"),
            &ChannelKey::new("discord", "main"),
        )
    })
    .await
    .expect("enqueue must be synchronous");

    assert_eq!(outcome, EnqueueOutcome::Full);
}

#[test]
fn closed_queue_has_a_distinct_shutdown_outcome() {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    drop(receiver);

    assert_eq!(
        try_enqueue(
            &sender,
            message("closed"),
            &ChannelKey::new("discord", "main")
        ),
        EnqueueOutcome::Closed
    );
}
