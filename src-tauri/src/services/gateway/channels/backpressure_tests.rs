use super::backpressure::{try_enqueue, EnqueueOutcome};
use super::InboundMessage;
use crate::services::gateway::refusal_audit::{self, RefusalAudit};
use crate::services::gateway::types::ChannelKey;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

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
    let (audit, receiver) = RefusalAudit::channel();
    let blocked = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Arc::clone(&blocked);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let worker = tokio::spawn(refusal_audit::run_with_writer_for_test(
        receiver,
        move |_, _| {
            let _ = started_tx.send(());
            let (lock, ready) = &*release;
            let mut released = lock.lock().unwrap();
            while !*released {
                released = ready.wait(released).unwrap();
            }
            true
        },
    ));
    assert_eq!(
        audit.try_record(ChannelKey::new("discord", "main"), "gateway_busy"),
        refusal_audit::RefusalAuditOutcome::Queued
    );
    started_rx.await.unwrap();

    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    sender.try_send(message("first")).unwrap();
    let key = ChannelKey::new("discord", "main");
    let (outcome_tx, outcome_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let outcome = try_enqueue(&sender, message("second"), &key, &audit);
        let _ = outcome_tx.send(outcome);
    });
    let outcome = outcome_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("a blocked audit writer must not block the channel loop");

    assert_eq!(outcome, EnqueueOutcome::Full);
    let (lock, ready) = &*blocked;
    *lock.lock().unwrap() = true;
    ready.notify_one();
    drop(_receiver);
    worker.await.unwrap();
}

#[test]
fn closed_queue_has_a_distinct_shutdown_outcome() {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    drop(receiver);
    let (audit, _audit_receiver) = RefusalAudit::channel();

    assert_eq!(
        try_enqueue(
            &sender,
            message("closed"),
            &ChannelKey::new("discord", "main"),
            &audit,
        ),
        EnqueueOutcome::Closed
    );
}

#[test]
fn refusal_audit_queue_is_bounded() {
    let (audit, _receiver) = RefusalAudit::channel();
    let key = ChannelKey::new("discord", "main");

    for _ in 0..refusal_audit::REFUSAL_AUDIT_CAPACITY {
        assert_eq!(
            audit.try_record(key.clone(), "gateway_busy"),
            refusal_audit::RefusalAuditOutcome::Queued
        );
    }
    assert_eq!(
        audit.try_record(key, "gateway_busy"),
        refusal_audit::RefusalAuditOutcome::Full
    );
    assert_eq!(audit.dropped(), 1);
}
