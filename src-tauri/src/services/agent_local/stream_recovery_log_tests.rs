use super::stream_recovery_log::StreamRecoveryLog;
use super::stream_recovery_record::*;
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

fn message(header: &StreamRecoveryHeader) -> super::types_message::AgentMessage {
    super::types_message::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: header.turn_id.clone(),
        role: "assistant".into(),
        content: "durable".into(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: Some(header.request_id.clone()),
        stream_part: Some("checkpoint".into()),
    }
}

#[tokio::test]
async fn event_append_and_semantic_rewrite_remain_readable() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Thinking {
        content: "work".into(),
    })
    .unwrap();
    log.stage_turn_ready().await.unwrap();
    let mut count = 0;
    super::stream_recovery_store::visit_records(&log.path(), |_| {
        count += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(count, 3);
    assert!(log.sticky_error().is_none());
    log.seal_and_remove().await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn pending_batch_replaces_uncommitted_deltas() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "delta".into(),
        phase: None,
    })
    .unwrap();
    log.stage_messages(vec![message(&header)]).await.unwrap();
    let mut kinds = Vec::new();
    super::stream_recovery_store::visit_records(&log.path(), |record| {
        kinds.push(record);
        Ok(())
    })
    .unwrap();
    assert_eq!(kinds.len(), 2);
    assert!(matches!(
        kinds[1],
        StreamRecoveryRecord::PendingMessage { .. }
    ));
    log.seal_and_remove().await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn oversized_record_is_sticky_and_cancels_the_owner() {
    let cancel = CancellationToken::new();
    let (log, lease) = StreamRecoveryLog::create(header(), cancel.clone())
        .await
        .unwrap();
    let result = log.record_event(RecoverableStreamEvent::Token {
        content: "x".repeat(super::stream_recovery_store::MAX_LINE_BYTES),
        phase: None,
    });
    assert!(result.is_err());
    assert!(cancel.is_cancelled());
    assert!(log.sticky_error().is_some());
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
}
