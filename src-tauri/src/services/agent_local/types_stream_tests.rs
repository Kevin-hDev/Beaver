use super::StreamEvent;
use super::super::types_message::AgentMessage;
use crate::services::agent_local::tool_result_contract::ToolResultStatus;
use crate::services::agent_local::types_interactive::AgentInteractiveChoiceKind;

#[test]
fn tool_result_serializes_readable_display_summary() {
    let event = StreamEvent::ToolResult {
        name: "load_skill".to_string(),
        content: "Skill loaded".to_string(),
        is_error: false,
        status: ToolResultStatus::Success,
        error: None,
        warnings: Vec::new(),
        truncated: false,
        display_summary: Some("context7-docs".to_string()),
        tool_call_index: 0,
        tool_call_id: Some("call-0".to_string()),
        resolved_path: None,
        domain: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
        start_line: None,
    };

    let serialized = serde_json::to_value(event).expect("stream event should serialize");

    assert_eq!(
        serialized["data"]["displaySummary"],
        serde_json::json!("context7-docs")
    );
    assert!(serialized.to_string().contains("context7-docs"));
    assert_eq!(serialized["data"]["toolCallIndex"], 0);
    assert_eq!(serialized["data"]["toolCallId"], "call-0");
    assert_eq!(serialized["data"]["status"], "success");
}

#[test]
fn tool_call_serializes_stable_identity_for_the_frontend() {
    let event = StreamEvent::ToolCall {
        name: "grep".to_string(),
        arguments: serde_json::json!({"pattern": "needle"}),
        tool_call_index: 2,
        tool_call_id: Some("call-2".to_string()),
        domain: None,
    };

    let serialized = serde_json::to_value(event).expect("serialize tool call");

    assert_eq!(serialized["data"]["toolCallIndex"], 2);
    assert_eq!(serialized["data"]["toolCallId"], "call-2");
    assert!(serialized["data"].get("tool_call_index").is_none());
}

#[test]
fn shell_output_serializes_live_progress_fields() {
    let event = StreamEvent::ToolOutput {
        tool_call_index: 3,
        content: "compilation...".to_string(),
        elapsed_ms: 1_250,
    };

    let serialized = serde_json::to_value(event).expect("serialize shell output");

    assert_eq!(serialized["event"], "toolOutput");
    assert_eq!(serialized["data"]["toolCallIndex"], 3);
    assert_eq!(serialized["data"]["elapsedMs"], 1_250);
}

#[test]
fn plan_approval_kind_reaches_the_frontend() {
    let event = StreamEvent::InteractiveChoiceRequest {
        session_id: "session-1".into(),
        id: "choice-1".into(),
        kind: AgentInteractiveChoiceKind::PlanApproval,
        questions: vec![],
        current_index: 0,
        total: 1,
    };

    let serialized = serde_json::to_value(event).expect("stream event should serialize");

    assert_eq!(
        serialized["data"]["kind"],
        serde_json::json!("plan_approval")
    );
}

#[test]
fn context_usage_serializes_for_the_live_ring() {
    let event = StreamEvent::ContextUsage {
        input_tokens: 120,
        output_tokens: 8,
        context_limit: 372_000,
        estimated: true,
        breakdown: Some(super::super::context_usage_buckets::RequestContextUsage {
            messages: 40,
            system_prompt: 80,
            reasoning_included: false,
            ..Default::default()
        }),
    };

    let serialized = serde_json::to_value(event).expect("serialize context usage");

    assert_eq!(serialized["event"], "contextUsage");
    assert_eq!(serialized["data"]["inputTokens"], 120);
    assert_eq!(serialized["data"]["outputTokens"], 8);
    assert_eq!(serialized["data"]["contextLimit"], 372_000);
    assert_eq!(serialized["data"]["estimated"], true);
    assert_eq!(serialized["data"]["breakdown"]["messages"], 40);
    assert_eq!(serialized["data"]["breakdown"]["systemPrompt"], 80);
    assert_eq!(serialized["data"]["breakdown"]["reasoningIncluded"], false);
}

#[test]
fn done_exposes_whether_tps_is_estimated() {
    let event = StreamEvent::Done {
        eval_count: Some(20),
        eval_duration_ns: 2_000_000_000,
        final_tps: 10.0,
        tps_estimated: true,
        prompt_tokens: Some(5),
        context_tokens: Some(25),
    };

    let serialized = serde_json::to_value(event).expect("serialize done");

    assert_eq!(serialized["data"]["tpsEstimated"], true);
    assert_eq!(serialized["data"]["evalDurationNs"], 2_000_000_000_u64);
}

#[test]
fn turn_lifecycle_events_expose_only_local_ids() {
    for event in [
        StreamEvent::TurnAdmitted {
            turn_id: "turn-local".into(),
            user_message_id: "user-local".into(),
            assistant_message_id: "assistant-local".into(),
        },
        StreamEvent::TurnCommitted {
            turn_id: "turn-local".into(),
            user_message_id: "user-local".into(),
            assistant_message_id: "assistant-local".into(),
        },
    ] {
        let serialized = serde_json::to_value(event).unwrap();
        let data = &serialized["data"];
        assert_eq!(data.as_object().unwrap().len(), 3);
        for forbidden in ["continuation", "replaySource", "credentialScope", "history"] {
            assert!(data.get(forbidden).is_none());
        }
    }
}

#[test]
fn session_snapshot_projects_only_positive_message_view_fields() {
    let envelope = super::super::session_view_test_support::responses_envelope(
        crate::services::reasoning_continuity::envelope::CompletionState::Complete,
    );
    let private = AgentMessage {
            id: "assistant-1".into(),
            turn_id: "turn-1".into(),
            role: "assistant".into(),
            content: "visible".into(),
            thinking: Some("visible thinking".into()),
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            replay_source: Some(envelope.source.clone()),
            continuation: Some(envelope),
            tool_activities: None,
            segments: None,
            files: Vec::new(),
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: Some(vec!["Visible".into()]),
            skill_ids: Some(vec!["private:skill:id".into()]),
            stream_run_id: None,
            stream_part: None,
        };
    let event = StreamEvent::SessionSnapshot {
        messages: super::super::session_view::messages(&[private]).unwrap(),
        token_count: 0,
    };

    let serialized = serde_json::to_value(event).unwrap();
    let message = &serialized["data"]["messages"][0];
    assert_eq!(message["content"], "visible");
    for forbidden in ["continuation", "replay_source", "replaySource", "skill_ids", "skillIds"] {
        assert!(message.get(forbidden).is_none(), "private field {forbidden}");
    }
}
