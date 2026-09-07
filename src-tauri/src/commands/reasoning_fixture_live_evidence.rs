use super::runner::TurnEvidence;
use crate::services::agent_local::types_session::AgentMessage;

pub(super) async fn require_tool_round(
    session_id: &str,
    evidence: &TurnEvidence,
) -> Result<(), String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    require_tool_round_in(&session.messages, evidence)
}

fn require_tool_round_in(messages: &[AgentMessage], evidence: &TurnEvidence) -> Result<(), String> {
    let calls = messages
        .iter()
        .filter(|message| message.turn_id == evidence.turn_id && message.role == "assistant")
        .flat_map(|message| message.tool_calls.as_deref().unwrap_or_default())
        .collect::<Vec<_>>();
    let expected = [
        ("fixture.write_note", serde_json::json!({"written": true})),
        ("fixture.read_note", serde_json::json!({"value": "fixture"})),
    ];
    let mut call_positions = [usize::MAX; 2];
    for (index, (name, expected_result)) in expected.into_iter().enumerate() {
        let call = calls
            .iter()
            .find(|call| call.function.name == name)
            .ok_or_else(|| "fixture tool evidence missing".to_string())?;
        call_positions[index] = calls
            .iter()
            .position(|candidate| candidate.function.name == name)
            .ok_or_else(|| "fixture tool evidence missing".to_string())?;
        let result = messages
            .iter()
            .find(|message| {
                message.turn_id == evidence.turn_id
                    && message.role == "tool"
                    && message.tool_name.as_deref() == Some(name)
                    && message.tool_call_id.as_deref() == Some(call.id.as_str())
            })
            .ok_or_else(|| "fixture tool result correlation missing".to_string())?;
        let value: serde_json::Value =
            serde_json::from_str(&result.content).map_err(|_| "fixture tool result invalid")?;
        if value != expected_result {
            return Err("fixture tool result invalid".into());
        }
    }
    if call_positions[0] >= call_positions[1] {
        return Err("fixture tool order invalid".into());
    }
    Ok(())
}

pub(super) async fn require_recall_without_tools(
    session_id: &str,
    evidence: &TurnEvidence,
    expected: &str,
) -> Result<(), String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    require_recall_in(&session.messages, &evidence.turn_id, expected)
}

fn require_recall_in(
    messages: &[crate::services::agent_local::types_session::AgentMessage],
    turn_id: &str,
    expected: &str,
) -> Result<(), String> {
    let messages = messages
        .iter()
        .filter(|message| message.turn_id == turn_id)
        .collect::<Vec<_>>();
    if messages.iter().any(|message| {
        message.role == "tool"
            || message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty())
    }) {
        return Err("fixture recall unexpectedly used a tool".into());
    }
    let content = messages
        .iter()
        .filter(|message| message.role == "assistant")
        .map(|message| message.content.as_str())
        .collect::<String>()
        .to_ascii_lowercase();
    answer_matches(&content, expected)
        .then_some(())
        .ok_or_else(|| "fixture recall failed".into())
}

pub(super) fn answer_matches(content: &str, expected: &str) -> bool {
    content
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .eq(expected.split_whitespace().map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::super::support::{select_specs, vision_capable};

    #[test]
    fn tool_proof_requires_success_value_ids_and_same_turn() {
        use serde_json::json;
        let message = |role: &str, content: &str| -> super::AgentMessage {
            serde_json::from_value(json!({
                "id":"message", "turn_id":"turn", "role":role,
                "content":content, "files":[], "timestamp":"2026-09-07T00:00:00Z"
            }))
            .unwrap()
        };
        let mut assistant = message("assistant", "");
        assistant.tool_calls = Some(serde_json::from_value(json!([
            {"id":"write", "function":{"name":"fixture.write_note", "arguments":{"value":"fixture"}}},
            {"id":"read", "function":{"name":"fixture.read_note", "arguments":{}}}
        ])).unwrap());
        let mut write = message("tool", r#"{"written":true}"#);
        write.tool_name = Some("fixture.write_note".into());
        write.tool_call_id = Some("write".into());
        let mut read = message("tool", r#"{"value":"fixture"}"#);
        read.tool_name = Some("fixture.read_note".into());
        read.tool_call_id = Some("read".into());
        let valid = vec![assistant, write, read];
        let evidence = super::TurnEvidence {
            turn_id: "turn".into(),
            request_id: "request".into(),
            payload_count: 2,
            reasoning_event_count: 1,
        };
        assert!(super::require_tool_round_in(&valid, &evidence).is_ok());
        for mutation in 0..5 {
            let mut invalid = valid.clone();
            match mutation {
                0 => invalid[2].tool_call_id = Some("wrong".into()),
                1 => invalid[2].turn_id = "previous".into(),
                2 => invalid[2].content = r#"{"value":"wrong"}"#.into(),
                3 => invalid[1].content = "tool failed".into(),
                _ => invalid[0].tool_calls.as_mut().unwrap().reverse(),
            }
            assert!(super::require_tool_round_in(&invalid, &evidence).is_err());
        }
    }

    #[test]
    fn vision_recall_accepts_a_color_without_requiring_the_tool_value() {
        let message = serde_json::from_value(serde_json::json!({
            "id":"answer", "turn_id":"turn", "role":"assistant",
            "content":"RED", "files":[], "timestamp":"2026-09-07T00:00:00Z"
        }))
        .unwrap();
        assert!(super::require_recall_in(&[message], "turn", "red").is_ok());
    }

    #[test]
    fn answer_checks_reject_negations_substrings_and_wrong_quadrant_order() {
        for invalid in ["NOT_RED", "not red", "red blue", "infrared"] {
            assert!(!super::answer_matches(invalid, "red"));
        }
        assert!(super::answer_matches(
            "Red, green, blue, yellow.",
            "red green blue yellow"
        ));
        assert!(!super::answer_matches(
            "red blue green yellow",
            "red green blue yellow"
        ));
    }

    #[test]
    fn new_vision_specs_are_selected_with_their_effective_mode() {
        let selected = select_specs(
            Some("zai,ollama"),
            Some("glm-5.3-flash,glm-5.3-flash:cloud"),
            Some("low"),
        )
        .unwrap();
        assert!(selected.iter().all(|spec| vision_capable(spec)));
        assert_eq!(selected[0].mode, "low");
        assert_eq!(selected[1].mode, "low");
    }
}
