use super::stream_recovery_record::*;

fn header() -> StreamRecoveryHeader {
    StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: uuid::Uuid::new_v4().to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    }
}

#[test]
fn header_and_nested_payload_reject_unknown_fields() {
    let record = StreamRecoveryRecord::Header(header());
    let mut value = serde_json::to_value(record).unwrap();
    value["data"]["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<StreamRecoveryRecord>(value).is_err());

    let event = StreamRecoveryRecord::Event {
        sequence: 1,
        event: RecoverableStreamEvent::ToolCall(RecoverableToolCall {
            name: "bash".into(),
            arguments: serde_json::json!({}),
            tool_call_index: 0,
            tool_call_id: None,
            domain: None,
            extra_content: None,
        }),
    };
    let mut value = serde_json::to_value(event).unwrap();
    value["data"]["event"]["data"]["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<StreamRecoveryRecord>(value).is_err());
}

#[test]
fn header_validates_every_identifier_and_version() {
    let valid = header();
    assert!(validate_header(&valid).is_ok());
    let mut invalid = valid;
    invalid.request_id = "../bad".into();
    assert_eq!(
        validate_header(&invalid),
        Err("stream_recovery_invalid".into())
    );
}
