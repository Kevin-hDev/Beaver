use super::types_ollama::ChatMessage;
use super::types_tools::ToolFollowUp;

const MAX_FOLLOW_UP_BYTES: usize = 16 * 1024;

#[derive(Default)]
pub struct ToolExecutionOutcome {
    pub compressed: bool,
    follow_ups: Vec<ToolFollowUp>,
}

impl ToolExecutionOutcome {
    pub fn with_compressed(compressed: bool) -> Self {
        Self {
            compressed,
            follow_ups: Vec::new(),
        }
    }

    pub fn record(&mut self, follow_up: ToolFollowUp) {
        if follow_up != ToolFollowUp::None {
            self.follow_ups.push(follow_up);
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.compressed |= other.compressed;
        self.follow_ups.extend(other.follow_ups);
    }

    pub fn apply_follow_ups(self, messages: &mut [ChatMessage]) -> Result<bool, String> {
        let mut stop = false;
        for follow_up in self.follow_ups {
            match follow_up {
                ToolFollowUp::None => {}
                ToolFollowUp::UserMessage(content) => {
                    append_to_tool(messages, "User follow-up", &content);
                }
                ToolFollowUp::SystemMessage(content) => {
                    append_to_tool(messages, "System follow-up", &content);
                }
                ToolFollowUp::Stop => stop = true,
            }
        }
        Ok(stop)
    }
}

fn append_to_tool(messages: &mut [ChatMessage], label: &str, content: &str) {
    let Some(tool) = messages
        .iter_mut()
        .rev()
        .find(|message| message.role == "tool")
    else {
        log::warn!("tool_follow_up_without_tool_message");
        return;
    };
    if content.is_empty() {
        return;
    }
    let content = bounded_prefix(content, MAX_FOLLOW_UP_BYTES);
    tool.content.push_str("\n\n");
    tool.content.push_str(label);
    tool.content.push_str(":\n");
    tool.content.push_str(content);
}

fn bounded_prefix(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let end = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= max_bytes)
        .last()
        .unwrap_or(0);
    &value[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_follow_ups_are_appended_after_tool_messages() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::UserMessage("User answer".into()));
        outcome.record(ToolFollowUp::SystemMessage("Backend state".into()));
        let mut messages = vec![ChatMessage::tool("Receipt".into(), None, None)];

        assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
        assert_eq!(messages.len(), 1);
        assert!(messages[0].content.contains("User answer"));
        assert!(messages[0].content.contains("Backend state"));
    }

    #[test]
    fn stop_follow_up_ends_the_batch_without_fabricating_a_message() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::Stop);
        let mut messages = Vec::new();

        assert!(outcome.apply_follow_ups(&mut messages).unwrap());
        assert!(messages.is_empty());
    }

    #[test]
    fn oversized_follow_up_is_bounded_without_failing_a_large_tool_result() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::UserMessage("é".repeat(MAX_FOLLOW_UP_BYTES)));
        let mut messages = vec![ChatMessage::tool("x".repeat(MAX_FOLLOW_UP_BYTES * 2), None, None)];

        assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
        assert!(messages[0].content.ends_with('é'));
        assert!(messages[0].content.len() <= MAX_FOLLOW_UP_BYTES * 3 + 32);
    }

    #[test]
    fn follow_up_without_tool_is_ignored_without_failing_the_turn() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::UserMessage("answer".into()));

        assert!(!outcome.apply_follow_ups(&mut []).unwrap());
    }
}
