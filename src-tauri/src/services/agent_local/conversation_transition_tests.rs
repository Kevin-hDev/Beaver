use chrono::Utc;

use super::conversation_transition::{self, ContinuityBarrier};
use super::types_message::AgentMessage;
use super::types_session::AgentSession;
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

#[test]
fn model_change_keeps_visible_history_but_blocks_native_envelopes() {
    let mut session = fixture_session();
    session.messages = complete_turn(
        "old",
        "visible answer",
        Some(envelope(RouteId::Ollama, "model-a", "opaque-native")),
    );

    let result = conversation_transition::for_target(&session, &target("model-b"));

    assert_eq!(result.barrier, Some(ContinuityBarrier::Model));
    assert_eq!(result.compatible_suffix_start, session.messages.len());
    assert_eq!(session.messages[1].content, "visible answer");
    assert!(result.replayable_message_indexes.is_empty());
}

#[test]
fn legacy_turn_without_provenance_becomes_a_replay_boundary() {
    let mut session = fixture_session();
    session.messages = complete_turn("legacy", "visible legacy answer", None);

    let result = conversation_transition::for_target(&session, &target("model-a"));

    assert!(result.barrier.is_some());
    assert_eq!(result.compatible_suffix_start, 2);
    assert!(result.replayable_message_indexes.is_empty());
}

#[test]
fn credential_and_mode_changes_are_explicit_barriers() {
    let mut session = fixture_session();
    session.messages = complete_turn(
        "old",
        "visible answer",
        Some(envelope(RouteId::Ollama, "model-a", "opaque-native")),
    );
    let mut credential_target = target("model-a");
    credential_target.credential_scope = CredentialScope::authenticated("scope-b").unwrap();
    credential_target.route_id = RouteId::OpenAi;
    let credential = conversation_transition::for_target(&session, &credential_target);
    assert_eq!(credential.barrier, Some(ContinuityBarrier::Route));

    let mut mode_target = target("model-a");
    mode_target.reasoning_mode = ReasoningModeId::High;
    let mode = conversation_transition::for_target(&session, &mode_target);
    assert_eq!(mode.barrier, Some(ContinuityBarrier::Mode));
}

#[test]
fn incompatible_provenance_discards_its_entire_user_assistant_tool_turn() {
    for source_index in 0..4 {
        let mut session = fixture_session();
        session.messages = complete_turn(
            "old-a",
            "old A",
            Some(envelope(RouteId::Ollama, "model-a", "opaque-old-a")),
        );
        let mut incompatible_turn = vec![
            message("user-b", "turn-b", "user", "question B"),
            message("assistant-tool-b", "turn-b", "assistant", "calling tool"),
            message("tool-b", "turn-b", "tool", "tool result"),
            message("assistant-b", "turn-b", "assistant", "answer B"),
        ];
        incompatible_turn[source_index].replay_source =
            Some(envelope(RouteId::Ollama, "model-b", "unused").source);
        session.messages.extend(incompatible_turn);
        session.messages.extend(complete_turn(
            "new-a",
            "new A",
            Some(envelope(RouteId::Ollama, "model-a", "opaque-new-a")),
        ));

        let result = conversation_transition::for_target(&session, &target("model-a"));

        assert_eq!(result.barrier, Some(ContinuityBarrier::Model));
        assert_eq!(result.compatible_suffix_start, 6);
    }
}

#[test]
fn compacting_only_removes_complete_turns_and_marks_replaced_envelopes() {
    let mut session = fixture_session();
    session.messages = complete_turn(
        "complete",
        "answer",
        Some(envelope(RouteId::Ollama, "model-a", "opaque-native")),
    );
    session
        .messages
        .push(message("user-open", "turn-open", "user", "unfinished"));

    let compacted =
        super::conversation_compaction::compact_complete_turns(&mut session.messages, 0)
            .expect("only the completed turn is compacted");

    assert_eq!(compacted.removed_turns, 1);
    assert_eq!(compacted.replaced_envelopes.len(), 1);
    assert_eq!(
        compacted.replaced_envelopes[0].completion,
        CompletionState::Compacted
    );
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].turn_id, "turn-open");
}

#[test]
fn compaction_refuses_an_open_tool_chain() {
    let mut session = fixture_session();
    let mut assistant = message("assistant-open", "turn-open", "assistant", "");
    assistant.tool_calls = Some(vec![super::types_message::ToolCallRequest {
        id: "call-open".into(),
        extra_content: None,
        function: super::types_message::ToolCallRequestFunction {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "a"}),
        },
    }]);
    session.messages = vec![
        message("user-open", "turn-open", "user", "question"),
        assistant,
    ];
    let before = serde_json::to_vec(&session.messages).unwrap();

    let error = super::conversation_compaction::compact_complete_turns(&mut session.messages, 1)
        .expect_err("open chains cannot be compacted");

    assert_eq!(
        error,
        super::conversation_compaction::CompactionError::OpenTurn
    );
    assert_eq!(serde_json::to_vec(&session.messages).unwrap(), before);
}

#[test]
fn required_user_replay_falls_back_after_a_historical_assistant_loses_its_envelope() {
    let replay_target = anthropic_target(ContinuationUse::UserContinuation);
    let mut session = fixture_session();
    let mut missing = complete_turn("missing", "visible answer", None);
    missing[0].replay_source = Some(source_for(&replay_target));
    let mut complete = complete_turn(
        "complete",
        "visible answer",
        Some(anthropic_envelope(&replay_target)),
    );
    complete[0].replay_source = Some(source_for(&replay_target));
    session.messages.extend(missing);
    session.messages.extend(complete);

    let result = conversation_transition::for_target(&session, &replay_target);

    assert_eq!(result.barrier, Some(ContinuityBarrier::Fallback));
    assert_eq!(result.compatible_suffix_start, 2);
    assert_eq!(result.replayable_message_indexes, vec![3]);
}

#[test]
fn current_user_only_turn_is_not_mistaken_for_a_missing_capture() {
    let replay_target = anthropic_target(ContinuationUse::UserContinuation);
    let mut session = fixture_session();
    let mut complete = complete_turn(
        "complete",
        "visible answer",
        Some(anthropic_envelope(&replay_target)),
    );
    complete[0].replay_source = Some(source_for(&replay_target));
    session.messages.extend(complete);
    let mut current = message("user-current", "turn-current", "user", "new question");
    current.replay_source = Some(source_for(&replay_target));
    session.messages.push(current);

    let result = conversation_transition::for_target(&session, &replay_target);

    assert_eq!(result.barrier, None);
    assert_eq!(result.compatible_suffix_start, 0);
    assert_eq!(result.replayable_message_indexes, vec![1]);
}

#[test]
fn required_turn_with_one_complete_envelope_does_not_fallback() {
    let replay_target = anthropic_target(ContinuationUse::UserContinuation);
    let mut session = fixture_session();
    let mut user = message("user", "turn", "user", "question");
    user.replay_source = Some(source_for(&replay_target));
    let assistant_with_capture = AgentMessage {
        continuation: Some(anthropic_envelope(&replay_target)),
        ..message("assistant-tool", "turn", "assistant", "calling tool")
    };
    session.messages = vec![
        user,
        assistant_with_capture,
        message("tool", "turn", "tool", "tool result"),
        message("assistant-final", "turn", "assistant", "visible answer"),
    ];

    let result = conversation_transition::for_target(&session, &replay_target);

    assert_eq!(result.barrier, None);
    assert_eq!(result.compatible_suffix_start, 0);
    assert_eq!(result.replayable_message_indexes, vec![1]);
}

#[test]
fn tool_continuation_with_missing_envelope_stays_fail_closed() {
    let replay_target = anthropic_target(ContinuationUse::ToolContinuation);
    let mut session = fixture_session();
    session.messages = complete_turn("missing", "visible answer", None);
    session.messages[0].replay_source = Some(source_for(&replay_target));

    let result = conversation_transition::for_target(&session, &replay_target);

    assert_ne!(result.barrier, Some(ContinuityBarrier::Fallback));
}

#[test]
fn optional_replay_does_not_create_fallback_for_missing_envelope() {
    let replay_target = ReplayTarget {
        route_id: RouteId::Zai,
        model_id: "glm-4.5-flash".into(),
        credential_scope: CredentialScope::authenticated("scope-a").unwrap(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    };
    let mut session = fixture_session();
    session.messages = complete_turn("missing", "visible answer", None);
    session.messages[0].replay_source = Some(source_for(&replay_target));

    let result = conversation_transition::for_target(&session, &replay_target);

    assert_ne!(result.barrier, Some(ContinuityBarrier::Fallback));
    assert_eq!(result.compatible_suffix_start, 0);
}

fn fixture_session() -> AgentSession {
    serde_json::from_value(serde_json::json!({
        "schema_version": 2,
        "id": "00000000-0000-4000-8000-000000000001",
        "name": "fixture",
        "created_at": Utc::now(),
        "model": "model-a",
        "provider": "ollama",
        "reasoning_mode": "auto",
        "messages": [],
        "accumulated_tokens": 0
    }))
    .unwrap()
}

fn target(model: &str) -> ReplayTarget {
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: model.into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

fn anthropic_target(continuation_use: ContinuationUse) -> ReplayTarget {
    ReplayTarget {
        route_id: RouteId::Anthropic,
        model_id: "claude-haiku-4-5-20251001".into(),
        credential_scope: CredentialScope::authenticated("scope-a").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use,
    }
}

fn source_for(target: &ReplayTarget) -> ReasoningSource {
    ReasoningSource {
        route_id: target.route_id,
        model_id: target.model_id.clone(),
        credential_scope: target.credential_scope.clone(),
        reasoning_mode: target.reasoning_mode,
    }
}

fn anthropic_envelope(target: &ReplayTarget) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::AnthropicMessagesV1,
        source_for(target),
        CompletionState::Complete,
        ContinuationState::AnthropicBlocks {
            blocks: vec![serde_json::json!({
                "type": "thinking",
                "thinking": "opaque",
                "signature": "AAE+/=="
            })],
        },
        Vec::new(),
    )
}

fn complete_turn(
    suffix: &str,
    answer: &str,
    continuation: Option<ReasoningEnvelope>,
) -> Vec<AgentMessage> {
    vec![
        message(
            &format!("user-{suffix}"),
            &format!("turn-{suffix}"),
            "user",
            "question",
        ),
        AgentMessage {
            continuation,
            ..message(
                &format!("assistant-{suffix}"),
                &format!("turn-{suffix}"),
                "assistant",
                answer,
            )
        },
    ]
}

fn envelope(route: RouteId, model: &str, thinking: &str) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: route,
            model_id: model.into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: thinking.into(),
        },
        Vec::new(),
    )
}

fn message(id: &str, turn_id: &str, role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: id.into(),
        turn_id: turn_id.into(),
        role: role.into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
