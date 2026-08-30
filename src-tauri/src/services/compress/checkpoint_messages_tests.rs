use super::checkpoint_messages::SelectedCheckpointMessage;
use super::checkpoint_selection::{select, CheckpointSelectionLimits};
use crate::services::agent_local::types_session::AgentMessage;

pub(super) fn message(turn: &str, role: &str, content: impl Into<String>) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: turn.to_string(),
        role: role.to_string(),
        content: content.into(),
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

pub(super) fn limits(user: u32, assistant: u32) -> CheckpointSelectionLimits {
    CheckpointSelectionLimits {
        user_tokens: user,
        assistant_tokens: assistant,
        tool_tokens: 20_000,
        tool_tokens_per_result: 4_000,
        max_tool_events: 100,
        total_tokens: user.saturating_add(assistant).saturating_add(20_000),
    }
}

#[test]
fn keeps_more_than_two_turns_and_restores_chronological_order() {
    let source = (0..4)
        .flat_map(|index| {
            let turn = format!("turn-{index}");
            [
                message(&turn, "user", format!("u{index}")),
                message(&turn, "assistant", format!("a{index}")),
            ]
        })
        .collect::<Vec<_>>();

    for budget in [5_000, 20_000] {
        let selected = select(&source, limits(budget, budget)).unwrap();
        let contents = selected
            .messages
            .iter()
            .map(|item| item.message().content.as_str())
            .collect::<Vec<_>>();
        assert_eq!(contents, ["u0", "a0", "u1", "a1", "u2", "a2", "u3", "a3"]);
        for item in &selected.messages {
            let SelectedCheckpointMessage::Exact {
                source_index,
                message,
            } = item
            else {
                panic!("small messages stay exact")
            };
            assert_eq!(
                serde_json::to_vec(message).unwrap(),
                serde_json::to_vec(&source[*source_index]).unwrap()
            );
        }
    }
}

#[test]
fn oversized_old_user_becomes_a_structured_excerpt_with_a_new_id() {
    let source = vec![
        message("old", "user", "x".repeat(120_000)),
        message("old", "assistant", "answer"),
        message("new", "user", "current"),
    ];
    let selected = select(&source, limits(5_000, 5_000)).unwrap();
    let excerpt = selected
        .messages
        .iter()
        .find_map(|item| match item {
            SelectedCheckpointMessage::TruncatedUserExcerpt {
                source_message_id,
                message,
                ..
            } => Some((source_message_id, message)),
            _ => None,
        })
        .expect("truncated user excerpt");

    assert_eq!(excerpt.0, &source[0].id);
    assert_ne!(excerpt.1.id, source[0].id);
    assert!(excerpt.1.content.contains("[user message excerpt]"));
    assert!(super::token_estimate::estimate_checkpoint_message_tokens(excerpt.1) <= 5_000);
}

#[test]
fn active_turn_is_atomic_and_never_truncated() {
    let source = vec![message("active", "user", "z".repeat(24_000))];
    let mut small = limits(1_000, 1_000);
    small.total_tokens = 5_000;
    assert_eq!(
        select(&source, small).unwrap_err(),
        crate::services::agent_local::context_capacity_error::CODE
    );

    let selected = select(&source, limits(5_000, 5_000)).unwrap();
    assert!(matches!(
        selected.messages[0],
        SelectedCheckpointMessage::Exact { .. }
    ));
    assert_eq!(selected.messages[0].message().id, source[0].id);
}

#[test]
fn indivisible_oversized_assistant_is_omitted_whole() {
    let source = vec![
        message("old", "user", "question"),
        message("old", "assistant", "r".repeat(40_000)),
        message("active", "user", "current"),
    ];
    let selected = select(&source, limits(5_000, 1_000)).unwrap();
    assert!(!selected
        .messages
        .iter()
        .any(|item| item.message().id == source[1].id));
}

#[test]
fn selected_user_without_assistant_survives_document_assembly() {
    let source = vec![
        message("old", "user", "keep this exact user intent"),
        message("old", "assistant", "r".repeat(40_000)),
        message("active", "user", "current work"),
    ];
    let selected = select(&source, limits(5_000, 1_000)).unwrap();
    let assembled = super::checkpoint_document::assemble(
        &selected.messages,
        Some("active"),
        None,
        &[],
        super::profile_types::CompressionTrigger::Explicit,
    )
    .unwrap();

    let checkpoint = assembled
        .iter()
        .find(|message| {
            message.message_kind
                == Some(crate::services::agent_local::types_message::AgentMessageKind::CompressionCheckpoint)
        })
        .expect("checkpoint");
    assert!(checkpoint.content.contains("retained_user_messages"));
    assert!(checkpoint.content.contains("keep this exact user intent"));
    assert!(checkpoint.content.contains("source_message_id"));
    assert!(!checkpoint.content.contains("tool_activities"));
    assert!(!checkpoint.content.contains("continuation"));
    assert_eq!(assembled.last().unwrap().content, "current work");
}

#[test]
fn selected_assistant_without_user_survives_document_assembly() {
    let source = vec![
        message("old", "user", "q".repeat(40_000)),
        message("old", "assistant", "keep this exact assistant answer"),
        message("active", "user", "current work"),
    ];
    let selected = select(&source, limits(0, 5_000)).unwrap();
    let assembled = super::checkpoint_document::assemble(
        &selected.messages,
        Some("active"),
        None,
        &[],
        super::profile_types::CompressionTrigger::Explicit,
    )
    .unwrap();

    let checkpoint = assembled
        .iter()
        .find(|message| {
            message.message_kind
                == Some(crate::services::agent_local::types_message::AgentMessageKind::CompressionCheckpoint)
        })
        .expect("checkpoint");
    assert!(checkpoint.content.contains("retained_assistant_messages"));
    assert!(checkpoint
        .content
        .contains("keep this exact assistant answer"));
    assert!(checkpoint.content.contains(&source[1].id));
    assert_eq!(assembled.last().unwrap().content, "current work");
}
