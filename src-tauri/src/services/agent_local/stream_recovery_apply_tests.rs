use super::stream_recovery_apply::{recover_session, OwnerTerminal, StreamRecoveryMode};
use super::stream_recovery_log::StreamRecoveryLog;
use super::stream_recovery_record::*;
use super::types_message::AgentMessage;
use tokio_util::sync::CancellationToken;

async fn session_and_header() -> (super::types_session::AgentSession, StreamRecoveryHeader) {
    let mut session = super::session_store::create_full(
        "Recovery apply",
        "model",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let header = StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: process_instance_id().into(),
        session_id: session.id.clone(),
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    };
    session.messages.push(user_message(&header));
    super::session_store::save(&session).await.unwrap();
    (session, header)
}

pub(super) fn user_message(header: &StreamRecoveryHeader) -> AgentMessage {
    AgentMessage {
        id: header.user_message_id.clone(),
        turn_id: header.turn_id.clone(),
        role: "user".into(),
        content: "continue".into(),
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
        stream_run_id: None,
        stream_part: None,
    }
}

fn terminal_assistant(turn_id: &str, content: &str) -> AgentMessage {
    let mut message = user_message(&StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: process_instance_id().into(),
        session_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: turn_id.into(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    });
    message.role = "assistant".into();
    message.content = content.into();
    message
}

#[tokio::test]
async fn stream_recovery_apply_is_idempotent_and_closes_pending_tools() {
    let (session, header) = session_and_header().await;
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Thinking {
        content: "visible work".into(),
    })
    .unwrap();
    log.record_event(RecoverableStreamEvent::ToolCall(RecoverableToolCall {
        name: "bash".into(),
        arguments: serde_json::json!({"command": "sleep 30"}),
        tool_call_index: 0,
        tool_call_id: Some("call-1".into()),
        domain: None,
        extra_content: None,
    }))
    .unwrap();

    recover_session(
        &session.id,
        StreamRecoveryMode::Owner {
            request_id: &header.request_id,
            terminal: OwnerTerminal::Failed {
                code: "stream_error",
            },
        },
    )
    .await
    .unwrap();
    let first = super::session_store::get(&session.id).await.unwrap();
    assert!(first.messages.iter().any(|message| {
        message.thinking.as_deref() == Some("visible work")
            && message.stream_run_id.as_deref() == Some(&header.request_id)
    }));
    assert!(first
        .messages
        .iter()
        .any(|message| message.content.contains("tool_interrupted")));
    let count = first.messages.len();

    drop(log);
    drop(lease);
    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    assert_eq!(super::session_store::get(&session.id).await.unwrap().messages.len(), count);
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stream_recovery_apply_ignores_a_live_non_owner_log() {
    let (session, header) = session_and_header().await;
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "not yet reclaimable".into(),
        phase: None,
    })
    .unwrap();

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    assert_eq!(super::session_store::get(&session.id).await.unwrap().messages.len(), 1);
    super::stream_recovery_store::remove(log.path()).await.unwrap();
    drop(lease);
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stream_recovery_apply_discards_run_a_after_durable_turn_b() {
    let (mut session, mut header) = session_and_header().await;
    header.process_instance_id = uuid::Uuid::new_v4().to_string();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "late A".into(),
        phase: None,
    })
    .unwrap();
    session.messages.push(terminal_assistant(&header.turn_id, "A complete"));
    let turn_b = uuid::Uuid::new_v4().to_string();
    let mut user_b = user_message(&header);
    user_b.id = uuid::Uuid::new_v4().to_string();
    user_b.turn_id = turn_b.clone();
    user_b.content = "B".into();
    session.messages.push(user_b);
    session.messages.push(terminal_assistant(&turn_b, "B complete"));
    super::session_store::save(&session).await.unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let recovered = super::session_store::get(&session.id).await.unwrap();
    assert!(!recovered.messages.iter().any(|message| message.content == "late A"));
    assert!(super::stream_recovery_store_discovery::session_paths(&session.id)
        .await
        .unwrap()
        .is_empty());
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stream_recovery_apply_quarantines_corruption_without_blocking_the_session() {
    let (session, header) = session_and_header().await;
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Thinking {
        content: "before corruption".into(),
    })
    .unwrap();
    std::io::Write::write_all(
        &mut std::fs::OpenOptions::new().append(true).open(log.path()).unwrap(),
        b"{broken}\n",
    )
    .unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    assert_eq!(super::session_store::get(&session.id).await.unwrap().messages.len(), 1);
    let quarantine = super::stream_recovery_store::root()
        .join("quarantine")
        .join(&session.id);
    assert_eq!(std::fs::read_dir(&quarantine).unwrap().count(), 1);
    super::session_store::delete_one(&session.id).await.unwrap();
    assert!(!quarantine.exists());
}

#[tokio::test]
async fn stream_recovery_apply_rejects_a_replaced_subagent_owner() {
    let (mut session, mut header) = session_and_header().await;
    let stale_run = uuid::Uuid::new_v4().to_string();
    header.subagent_owner = Some(StreamRecoveryOwner {
        run_id: stale_run,
        execution_id: uuid::Uuid::new_v4().to_string(),
    });
    session.subagent_run_id = Some(uuid::Uuid::new_v4().to_string());
    super::session_store::save(&session).await.unwrap();
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "stale child".into(),
        phase: None,
    })
    .unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let recovered = super::session_store::get(&session.id).await.unwrap();
    assert!(!recovered.messages.iter().any(|message| message.content == "stale child"));
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stream_recovery_apply_keeps_the_journal_when_session_capacity_is_full() {
    let (mut session, header) = session_and_header().await;
    append_tool_step(&mut session.messages, &header.turn_id, "wide-a", 0);
    append_tool_result(&mut session.messages, &header.turn_id, "wide-a", 0);
    append_tool_result(&mut session.messages, &header.turn_id, "wide-b", 1);
    for index in 1..999 {
        let call_id = format!("call-{index}");
        append_tool_step(&mut session.messages, &header.turn_id, &call_id, index);
        append_tool_result(&mut session.messages, &header.turn_id, &call_id, index);
    }
    assert_eq!(session.messages.len(), super::session_limits::MAX_MESSAGES_PER_SESSION);
    super::session_store::save(&session).await.unwrap();
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "cannot fit".into(),
        phase: None,
    })
    .unwrap();
    drop(log);
    drop(lease);

    assert_eq!(
        recover_session(&session.id, StreamRecoveryMode::StaleOnly)
            .await
            .unwrap_err(),
        "session_capacity_reached"
    );
    assert_eq!(
        super::stream_recovery_store_discovery::session_paths(&session.id)
            .await
            .unwrap()
            .len(),
        1
    );
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stream_recovery_apply_attaches_a_result_to_its_durable_tool_call() {
    let (mut session, header) = session_and_header().await;
    append_tool_step(&mut session.messages, &header.turn_id, "call-durable", 0);
    super::session_store::save(&session).await.unwrap();
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    let result = super::types_tools::ToolResult::ok("durable result").persistence_snapshot(
        "tool-0",
        0,
        Some("call-durable"),
        None,
        None,
        Vec::new(),
    );
    log.record_event(RecoverableStreamEvent::ToolResult(result))
        .unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let recovered = super::session_store::get(&session.id).await.unwrap();
    assert_eq!(
        recovered
            .messages
            .iter()
            .filter(|message| message.tool_call_id.as_deref() == Some("call-durable"))
            .count(),
        1
    );
    assert!(recovered.messages.iter().any(|message| message.content == "durable result"));
    super::session_store::delete_one(&session.id).await.unwrap();
}

fn append_tool_step(messages: &mut Vec<AgentMessage>, turn_id: &str, call_id: &str, index: usize) {
    let mut assistant = terminal_assistant(turn_id, "");
    let mut calls = vec![tool_call(call_id, index)];
    if call_id == "wide-a" {
        calls.push(tool_call("wide-b", index + 1));
    }
    assistant.tool_calls = Some(calls);
    messages.push(assistant);
}

fn tool_call(call_id: &str, index: usize) -> super::types_message::ToolCallRequest {
    super::types_message::ToolCallRequest {
        id: call_id.into(),
        extra_content: None,
        function: super::types_message::ToolCallRequestFunction {
            name: format!("tool-{index}"),
            arguments: serde_json::json!({}),
        },
    }
}

fn append_tool_result(messages: &mut Vec<AgentMessage>, turn_id: &str, call_id: &str, index: usize) {
    let mut result = terminal_assistant(turn_id, "ok");
    result.role = "tool".into();
    result.tool_name = Some(format!("tool-{index}"));
    result.tool_call_id = Some(call_id.into());
    messages.push(result);
}
