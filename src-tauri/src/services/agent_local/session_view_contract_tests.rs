use serde_json::json;

use crate::models::agent_session_contract::{ReasoningReplayStatus, VisibleMessageInput};
use crate::services::reasoning_continuity::envelope::CompletionState;

use super::session_view_test_support::{fixture_session, responses_envelope};

#[test]
fn response_items_never_cross_the_visible_session_boundary() {
    let mut session = fixture_session();
    let assistant = session.messages.get_mut(1).expect("assistant fixture");
    assistant.content = "Le nom encrypted_content reste lisible.".into();
    assistant.thinking = Some("thinking affiché".into());
    assistant.continuation = Some(responses_envelope(CompletionState::Complete));
    assistant.tool_calls = Some(vec![super::types_message::ToolCallRequest {
        id: "provider-call-42".into(),
        extra_content: Some(json!({
            "thoughtSignature": "opaque-signature",
            "encrypted_content": "opaque-secret"
        })),
        function: super::types_message::ToolCallRequestFunction {
            name: "inspect".into(),
            arguments: json!({
                "encrypted_content": "argument visible",
                "previous_response_id": "texte contrôlé"
            }),
        },
    }]);

    let view = super::session_view::from_session(&session).expect("visible view");
    let assistant_view = &view.messages[1];
    let serialized = serde_json::to_string(&view).expect("serialize visible view");

    assert_eq!(assistant_view.content, "Le nom encrypted_content reste lisible.");
    assert_eq!(assistant_view.reasoning_replay_status, ReasoningReplayStatus::Preserved);
    assert_eq!(assistant_view.tool_calls.as_ref().unwrap()[0].id, "provider-call-42");
    assert_eq!(
        assistant_view.tool_calls.as_ref().unwrap()[0].function.arguments,
        json!({
            "encrypted_content": "argument visible",
            "previous_response_id": "texte contrôlé"
        })
    );
    assert!(!serialized.contains("opaque-signature"));
    assert!(!serialized.contains("opaque-secret"));
    let value = serde_json::to_value(view).expect("serialize visible structure");
    assert!(value.pointer("/messages/1/continuation").is_none());
    assert!(value.pointer("/messages/1/tool_calls/0/extra_content").is_none());
}

#[test]
fn replay_status_comes_from_the_envelope_not_display_thinking() {
    let mut session = fixture_session();
    session.messages[1].thinking = Some("visible mais non rejouable".into());
    session.messages[1].continuation = None;
    assert_eq!(
        super::session_view::from_session(&session).unwrap().messages[1].reasoning_replay_status,
        ReasoningReplayStatus::Unavailable
    );

    session.messages[1].continuation = Some(responses_envelope(CompletionState::Partial));
    assert_eq!(
        super::session_view::from_session(&session).unwrap().messages[1].reasoning_replay_status,
        ReasoningReplayStatus::Partial
    );
    session.messages[1].continuation = Some(responses_envelope(CompletionState::Compacted));
    assert_eq!(
        super::session_view::from_session(&session).unwrap().messages[1].reasoning_replay_status,
        ReasoningReplayStatus::Compacted
    );

    let mut invalid = responses_envelope(CompletionState::Complete);
    invalid.schema_version = u16::MAX;
    session.messages[1].continuation = Some(invalid);
    assert_eq!(
        super::session_view::from_session(&session).unwrap().messages[1].reasoning_replay_status,
        ReasoningReplayStatus::Unavailable
    );
}

#[test]
fn visible_messages_keep_turn_order_and_provider_tool_ids() {
    let mut session = fixture_session();
    session.messages[0].turn_id = "turn-one".into();
    session.messages[1].turn_id = "turn-one".into();
    session.messages[1].tool_calls = Some(vec![super::types_message::ToolCallRequest {
        id: "provider-call-one".into(),
        extra_content: None,
        function: super::types_message::ToolCallRequestFunction {
            name: "read_file".into(),
            arguments: json!({"path":"fixture.txt"}),
        },
    }]);
    let mut tool = session.messages[1].clone();
    tool.id = "00000000-0000-4000-8000-000000000003".into();
    tool.role = "tool".into();
    tool.content = "résultat".into();
    tool.tool_calls = None;
    tool.tool_name = Some("read_file".into());
    tool.tool_call_id = Some("provider-call-one".into());
    let mut second_user = session.messages[0].clone();
    second_user.id = "00000000-0000-4000-8000-000000000004".into();
    second_user.turn_id = "turn-two".into();
    second_user.content = "Deuxième question".into();
    session.messages.push(tool);
    session.messages.push(second_user);

    let view = super::session_view::from_session(&session).expect("visible view");
    let ordered = view
        .messages
        .iter()
        .map(|message| (message.role.as_str(), message.turn_id.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        ordered,
        vec![
            ("user", "turn-one"),
            ("assistant", "turn-one"),
            ("tool", "turn-one"),
            ("user", "turn-two"),
        ]
    );
    assert_eq!(view.messages[1].tool_calls.as_ref().unwrap()[0].id, "provider-call-one");
    assert_eq!(view.messages[2].tool_call_id.as_deref(), Some("provider-call-one"));
}

#[test]
fn visible_shim_rejects_an_unbounded_tool_call_collection() {
    let calls = (0..=crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS)
        .map(|index| json!({
            "id": format!("call-{index}"),
            "function": {"name": "inspect", "arguments": {}}
        }))
        .collect::<Vec<_>>();
    let input: VisibleMessageInput = serde_json::from_value(json!({
        "id": "00000000-0000-4000-8000-000000000019",
        "role": "assistant",
        "content": "visible",
        "tool_calls": calls,
        "files": [],
        "timestamp": "2026-08-25T10:00:00Z"
    }))
    .expect("syntactically visible input");

    assert!(super::session_visible_input::into_message(input, "turn-visible".into()).is_err());
}
