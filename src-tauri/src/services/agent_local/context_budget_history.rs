use super::types_ollama::ChatMessage;

pub struct HistoryUnit {
    pub messages: Vec<ChatMessage>,
    pub is_tool_chain: bool,
    pub valid: bool,
}

pub fn atomic_units(messages: Vec<ChatMessage>) -> Vec<HistoryUnit> {
    let mut units = Vec::new();
    let mut messages = messages.into_iter().peekable();
    while let Some(message) = messages.next() {
        let expected = message
            .tool_calls
            .as_ref()
            .filter(|calls| !calls.is_empty())
            .map_or(0, Vec::len);
        if message.role == "assistant" && expected > 0 {
            let calls = message.tool_calls.clone().unwrap_or_default();
            let mut chain = vec![message];
            while chain.len() <= expected
                && messages.peek().is_some_and(|next| next.role == "tool")
            {
                chain.push(messages.next().expect("peeked tool message"));
            }
            let valid = chain.len() == expected + 1 && matching_tool_results(&calls, &chain[1..]);
            units.push(HistoryUnit {
                messages: chain,
                is_tool_chain: true,
                valid,
            });
        } else if message.role == "tool" {
            units.push(HistoryUnit {
                messages: vec![message],
                is_tool_chain: true,
                valid: false,
            });
        } else {
            units.push(HistoryUnit {
                messages: vec![message],
                is_tool_chain: false,
                valid: true,
            });
        }
    }
    units
}

fn matching_tool_results(
    calls: &[super::types_ollama::ToolCallOllama],
    results: &[ChatMessage],
) -> bool {
    calls.iter().zip(results).all(|(call, result)| {
        let id_matches = match (&call.id, &result.tool_call_id) {
            (Some(expected), Some(actual)) => expected == actual,
            (Some(_), None) => false,
            (None, _) => true,
        };
        let name_matches = result
            .tool_name
            .as_deref()
            .is_none_or(|name| name == call.function.name);
        id_matches && name_matches
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn groups_a_call_with_all_of_its_results() {
        let units = atomic_units(vec![assistant_call("call-1"), tool_result("call-1")]);

        assert_eq!(units.len(), 1);
        assert!(units[0].valid);
        assert!(units[0].is_tool_chain);
        assert_eq!(units[0].messages.len(), 2);
    }

    #[test]
    fn rejects_orphan_and_mismatched_results() {
        let orphan = atomic_units(vec![tool_result("call-1")]);
        let mismatch = atomic_units(vec![assistant_call("call-1"), tool_result("call-2")]);

        assert!(!orphan[0].valid);
        assert!(!mismatch[0].valid);
    }

    fn assistant_call(id: &str) -> ChatMessage {
        ChatMessage {
            role: "assistant".into(),
            content: String::new(),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some(id.into()),
                extra_content: None,
                function: ToolCallFunction { name: "grep".into(), arguments: serde_json::json!({}) },
            }]),
            ..Default::default()
        }
    }

    fn tool_result(id: &str) -> ChatMessage {
        ChatMessage {
            role: "tool".into(),
            content: "ok".into(),
            tool_name: Some("grep".into()),
            tool_call_id: Some(id.into()),
            ..Default::default()
        }
    }
}
