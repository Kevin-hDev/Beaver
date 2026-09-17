use super::stream_events::AgentEventEmitter;
use super::stream_recovery_log::StreamRecoveryLog;
use super::stream_recovery_record::{
    process_instance_id, RecoverableStreamEvent, StreamRecoveryHeader, StreamRecoveryRecord,
    STREAM_RECOVERY_VERSION,
};
use super::types_ollama::StreamEvent;
use tokio_util::sync::CancellationToken;

fn header() -> StreamRecoveryHeader {
    StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: process_instance_id().to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn stream_events_recovery_is_written_without_an_app_before_live_delivery() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    let emitter = AgentEventEmitter::test(header.session_id).with_recovery_log(log.clone());

    emitter
        .send(StreamEvent::Thinking {
            content: "durable first".into(),
            token_count: 2,
        })
        .unwrap();

    let mut events = Vec::new();
    super::stream_recovery_store::visit_records(&log.path(), |record| {
        if let StreamRecoveryRecord::Event { event, .. } = record {
            events.push(event);
        }
        Ok(())
    })
    .unwrap();
    assert!(matches!(
        events.as_slice(),
        [RecoverableStreamEvent::Thinking { content }] if content == "durable first"
    ));
    log.seal_and_remove().await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn stream_events_recovery_failure_cancels_and_fails_closed() {
    let cancel = CancellationToken::new();
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), cancel.clone())
        .await
        .unwrap();
    let emitter = AgentEventEmitter::test(header.session_id).with_recovery_log(log.clone());

    assert!(emitter
        .send(StreamEvent::Thinking {
            content: "x".repeat(super::stream_recovery_store::MAX_LINE_BYTES),
            token_count: 0,
        })
        .is_err());
    assert!(cancel.is_cancelled());
    assert!(log.sticky_error().is_some());
    super::stream_recovery_store::remove(log.path())
        .await
        .unwrap();
    drop(lease);
}

#[test]
fn retry_indicator_has_one_recovery_projection() {
    let event = StreamEvent::RetryIndicator {
        reason_key: "agentLocal.retry.server".into(),
        attempt: 2,
        max_attempts: 10,
    };
    assert!(matches!(
        RecoverableStreamEvent::from_stream_event(&event),
        Some(RecoverableStreamEvent::AttemptRestarted { attempt: 2, .. })
    ));
}
