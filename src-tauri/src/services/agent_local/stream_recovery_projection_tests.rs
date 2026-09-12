use super::stream_recovery_log::StreamRecoveryLog;
use super::stream_recovery_projection::from_path;
use super::stream_recovery_record::*;
use super::types_ollama::StreamEvent;
use tokio_util::sync::CancellationToken;

fn header() -> StreamRecoveryHeader {
    StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: process_instance_id().into(),
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
async fn stream_recovery_projection_keeps_only_the_last_retry_attempt_and_phase_order() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    for event in [
        StreamEvent::Thinking {
            content: "discarded".into(),
            token_count: 1,
        },
        StreamEvent::RetryIndicator {
            reason_key: "agentLocal.retry.server".into(),
            attempt: 2,
            max_attempts: 10,
        },
        StreamEvent::ContentPhase {
            phase: super::types_stream::TokenPhase::Work,
        },
        StreamEvent::Token {
            content: "work".into(),
            token_count: 1,
            tps: 1.0,
            phase: Some(super::types_stream::TokenPhase::Work),
        },
        StreamEvent::ContentPhase {
            phase: super::types_stream::TokenPhase::Final,
        },
        StreamEvent::Token {
            content: "answer".into(),
            token_count: 1,
            tps: 1.0,
            phase: Some(super::types_stream::TokenPhase::Final),
        },
    ] {
        log.record_event(RecoverableStreamEvent::from_stream_event(&event).unwrap())
            .unwrap();
    }

    let projection = from_path(&log.path()).unwrap();
    let assistant = &projection.messages[0];
    assert_eq!(assistant.content, "workanswer");
    assert!(!assistant.thinking.as_deref().unwrap_or("").contains("discarded"));
    assert_eq!(assistant.segments.as_ref().unwrap().len(), 2);
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn stream_recovery_projection_restores_private_tool_data() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::ToolCall(RecoverableToolCall {
        name: "read_file".into(),
        arguments: serde_json::json!({"path": "log"}),
        tool_call_index: 0,
        tool_call_id: Some("call-1".into()),
        domain: Some("memory".into()),
        extra_content: Some(serde_json::json!({
            "google": {"thought_signature": "private"}
        })),
    }))
    .unwrap();
    let result = super::types_tools::ToolResult::partial("preview", ["warning"])
        .with_user_message("follow-up")
        .persistence_snapshot(
            "read_file",
            0,
            Some("call-1"),
            Some("log".into()),
            Some("memory".into()),
            Vec::new(),
        );
    log.record_event(RecoverableStreamEvent::ToolResult(result))
        .unwrap();

    let projection = from_path(&log.path()).unwrap();
    assert_eq!(projection.messages.len(), 2);
    assert_eq!(
        projection.messages[0].tool_calls.as_ref().unwrap()[0].extra_content,
        Some(serde_json::json!({
            "google": {"thought_signature": "private"}
        }))
    );
    assert!(projection.messages[1].content.contains("follow-up"));
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn stream_recovery_projection_rejects_an_incomplete_pending_batch() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    let message = super::stream_recovery_apply_tests::user_message(&header);
    log.append_with(
        |sequence| StreamRecoveryRecord::PendingMessage {
            sequence,
            batch_id: uuid::Uuid::new_v4().to_string(),
            position: 0,
            total: 2,
            message,
        },
        false,
    )
    .unwrap();

    assert!(matches!(
        from_path(&log.path()),
        Err(error) if error == "stream_recovery_invalid"
    ));
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
}

#[tokio::test]
async fn stream_recovery_projection_prefers_a_complete_pending_batch_and_turn_ready() {
    let header = header();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "obsolete delta".into(),
        phase: None,
    })
    .unwrap();
    let mut message = super::stream_recovery_apply_tests::user_message(&header);
    message.id = header.assistant_message_id.clone();
    message.role = "assistant".into();
    message.content = "exact checkpoint".into();
    message.stream_run_id = Some(header.request_id.clone());
    message.stream_part = Some("checkpoint".into());
    log.stage_messages(vec![message]).await.unwrap();
    log.stage_turn_ready().await.unwrap();

    let projection = from_path(&log.path()).unwrap();
    assert_eq!(projection.messages.len(), 1);
    assert_eq!(projection.messages[0].content, "exact checkpoint");
    assert_eq!(projection.messages[0].stream_part.as_deref(), Some("final"));
    assert!(projection.turn_ready);
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
}
