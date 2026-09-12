
use super::*;

#[tokio::test]
async fn stream_recovery_process_converts_a_worker_panic_and_restores_its_token() {
    use crate::services::agent_local::stream_recovery_log::StreamRecoveryLog;
    use crate::services::agent_local::stream_recovery_record::*;
    let mut session = crate::services::agent_local::session_store::create_full(
        "Panic recovery",
        "model",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id = uuid::Uuid::new_v4().to_string();
    let header = StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: process_instance_id().into(),
        session_id: session.id.clone(),
        request_id: request_id.clone(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    };
    session.messages.push(user_message(&header));
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let cancel = CancellationToken::new();
    let (log, lease) = StreamRecoveryLog::create(header, cancel.clone())
        .await
        .unwrap();
    log.record_event(RecoverableStreamEvent::Token {
        content: "visible before panic".into(),
        phase: None,
    })
    .unwrap();
    drop(log);
    drop(lease);
    let guarded = guard(async {
        panic!("private panic payload");
        #[allow(unreachable_code)]
        Ok(CompletedStreamTurn::compression(Vec::new()))
    })
    .await;

    let error = match finish(guarded, &session.id, &request_id, &cancel).await {
        Err(error) => error,
        Ok(_) => panic!("panic must become an error"),
    };
    assert_eq!(error, "stream_error");
    assert!(cancel.is_cancelled());
    assert!(
        crate::services::agent_local::session_store::get(&session.id)
            .await
            .unwrap()
            .messages
            .iter()
            .any(|message| message.content == "visible before panic")
    );
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

fn user_message(
    header: &crate::services::agent_local::stream_recovery_record::StreamRecoveryHeader,
) -> crate::services::agent_local::types_message::AgentMessage {
    crate::services::agent_local::types_message::AgentMessage {
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
