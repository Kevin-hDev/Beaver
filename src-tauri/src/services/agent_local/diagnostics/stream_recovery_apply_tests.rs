use super::stream_recovery_apply::{recover_session, OwnerTerminal, StreamRecoveryMode};
use super::stream_recovery_log::StreamRecoveryLog;
use super::stream_recovery_record::*;
use super::types_message::AgentMessage;
use tokio_util::sync::CancellationToken;

async fn session_and_header() -> (super::types_session::AgentSession, StreamRecoveryHeader) {
    let mut session =
        super::session_store::create_full("Recovery apply", "model", "openai", false, None)
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
    assert_eq!(
        super::session_store::get(&session.id)
            .await
            .unwrap()
            .messages
            .len(),
        count
    );
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn replay_after_saved_recovery_preserves_terminal_diagnostics() {
    let (session, mut header) = session_and_header().await;
    header.request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Thinking {
        content: "durable work".into(),
    })
    .unwrap();
    let path = log.path();
    let journal = std::fs::read(&path).unwrap();

    recover_session(
        &session.id,
        StreamRecoveryMode::Owner {
            request_id: &header.request_id,
            terminal: OwnerTerminal::Cancelled,
        },
    )
    .await
    .unwrap();
    let first = super::session_store::get(&session.id).await.unwrap();
    assert_eq!(first.diagnostic_runs[0].status, "cancelled");
    assert!(first.stream_failures.is_empty());
    let session_path = crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{}.json", session.id));
    let first_bytes = std::fs::read(&session_path).unwrap();

    drop(log);
    drop(lease);
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, journal).unwrap();
    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let replayed = super::session_store::get(&session.id).await.unwrap();

    let first_run = &first.diagnostic_runs[0];
    let replayed_run = &replayed.diagnostic_runs[0];
    assert_eq!(replayed_run.status, first_run.status);
    assert_eq!(replayed_run.error_type, first_run.error_type);
    assert_eq!(replayed_run.ended_at, first_run.ended_at);
    assert_eq!(replayed_run.events.len(), first_run.events.len());
    assert_eq!(replayed.stream_failures.len(), first.stream_failures.len());
    assert_eq!(replayed.updated_at, first.updated_at);
    assert_eq!(std::fs::read(session_path).unwrap(), first_bytes);
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
    assert_eq!(
        super::session_store::get(&session.id)
            .await
            .unwrap()
            .messages
            .len(),
        1
    );
    super::stream_recovery_store::remove(log.path())
        .await
        .unwrap();
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
    session
        .messages
        .push(terminal_assistant(&header.turn_id, "A complete"));
    let turn_b = uuid::Uuid::new_v4().to_string();
    let mut user_b = user_message(&header);
    user_b.id = uuid::Uuid::new_v4().to_string();
    user_b.turn_id = turn_b.clone();
    user_b.content = "B".into();
    session.messages.push(user_b);
    session
        .messages
        .push(terminal_assistant(&turn_b, "B complete"));
    super::session_store::save(&session).await.unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let recovered = super::session_store::get(&session.id).await.unwrap();
    assert!(!recovered
        .messages
        .iter()
        .any(|message| message.content == "late A"));
    assert!(
        super::stream_recovery_store_discovery::session_paths(&session.id)
            .await
            .unwrap()
            .is_empty()
    );
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
        &mut std::fs::OpenOptions::new()
            .append(true)
            .open(log.path())
            .unwrap(),
        b"{broken}\n",
    )
    .unwrap();
    drop(log);
    drop(lease);

    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    assert_eq!(
        super::session_store::get(&session.id)
            .await
            .unwrap()
            .messages
            .len(),
        1
    );
    let quarantine = super::stream_recovery_store::root()
        .join("quarantine")
        .join(&session.id);
    assert_eq!(std::fs::read_dir(&quarantine).unwrap().count(), 1);
    super::session_store::delete_one(&session.id).await.unwrap();
    assert!(!quarantine.exists());
}

#[tokio::test]
async fn claimed_journal_corruption_is_quarantined() {
    let (session, header) = session_and_header().await;
    let (log, lease) = StreamRecoveryLog::create(header, CancellationToken::new())
        .await
        .unwrap();
    let claimed = super::stream_recovery_store_discovery::claim(log.path())
        .await
        .unwrap();
    std::io::Write::write_all(
        &mut std::fs::OpenOptions::new()
            .append(true)
            .open(&claimed)
            .unwrap(),
        b"{broken}\n",
    )
    .unwrap();

    assert!(super::stream_recovery_apply::load_claimed(&claimed)
        .await
        .unwrap()
        .is_none());
    let quarantine = super::stream_recovery_store::root()
        .join("quarantine")
        .join(&session.id);
    assert_eq!(std::fs::read_dir(&quarantine).unwrap().count(), 1);
    drop(log);
    drop(lease);
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
    assert!(!recovered
        .messages
        .iter()
        .any(|message| message.content == "stale child"));
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
    assert_eq!(
        session.messages.len(),
        super::session_limits::MAX_MESSAGES_PER_SESSION
    );
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
    assert!(recovered
        .messages
        .iter()
        .any(|message| message.content == "durable result"));
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

fn append_tool_result(
    messages: &mut Vec<AgentMessage>,
    turn_id: &str,
    call_id: &str,
    index: usize,
) {
    let mut result = terminal_assistant(turn_id, "ok");
    result.role = "tool".into();
    result.tool_name = Some(format!("tool-{index}"));
    result.tool_call_id = Some(call_id.into());
    messages.push(result);
}

#[tokio::test]
async fn cancelled_delta_after_tool_checkpoints_keeps_committed_message_identities() {
    use super::conversation_journal::ConversationJournal;
    use super::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

    let (session, mut header) = session_and_header().await;
    header.request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        header.turn_id.clone(),
        header.user_message_id.clone(),
        header.assistant_message_id.clone(),
        header.request_id.clone(),
    )
    .unwrap();
    let (log, lease) = StreamRecoveryLog::create(header.clone(), CancellationToken::new())
        .await
        .unwrap();
    journal.attach_recovery(log.clone(), lease);
    for index in 0..2 {
        let call_id = format!("call-checkpoint-{index}");
        journal
            .persist_assistant_step(&ChatMessage::assistant(
                String::new(),
                None,
                None,
                None,
                Some(vec![ToolCallOllama {
                    id: Some(call_id.clone()),
                    function: ToolCallFunction {
                        name: "read_file".into(),
                        arguments: serde_json::json!({"path":"facts.txt"}),
                    },
                    extra_content: None,
                }]),
            ))
            .await
            .unwrap();
        journal
            .persist_tool_results(
                &[ChatMessage::tool(
                    "durable result".into(),
                    Some(call_id),
                    Some("read_file".into()),
                )],
                &[],
            )
            .await
            .unwrap();
    }
    let committed = super::session_store::get(&session.id)
        .await
        .unwrap()
        .messages;
    log.record_event(RecoverableStreamEvent::Token {
        content: "interrupted continuation".into(),
        phase: None,
    })
    .unwrap();
    recover_session(
        &session.id,
        StreamRecoveryMode::Owner {
            request_id: &header.request_id,
            terminal: OwnerTerminal::Cancelled,
        },
    )
    .await
    .unwrap();
    drop(journal);
    drop(log);
    recover_session(&session.id, StreamRecoveryMode::StaleOnly)
        .await
        .unwrap();
    let recovered = super::session_store::get(&session.id).await.unwrap();
    assert_eq!(recovered.messages.len(), committed.len() + 1);
    for (before, after) in committed.iter().zip(&recovered.messages) {
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(after).unwrap()
        );
    }
    let tail = recovered.messages.last().unwrap();
    assert_eq!(tail.content, "interrupted continuation");
    assert!(committed.iter().all(|message| message.id != tail.id));
    assert_eq!(
        recovered.diagnostic_runs.last().unwrap().status,
        "cancelled"
    );
    super::conversation_history_validation::validate(&recovered.messages).unwrap();
    super::session_store::delete_one(&session.id).await.unwrap();
}
