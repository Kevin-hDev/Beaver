use super::types_ollama::ChatMessage;
use super::types_tools::ToolFollowUp;

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

    pub fn apply_follow_ups(self, messages: &mut Vec<ChatMessage>) -> bool {
        let mut stop = false;
        for follow_up in self.follow_ups {
            match follow_up {
                ToolFollowUp::None => {}
                ToolFollowUp::UserMessage(content) => {
                    messages.push(message("user", content));
                }
                ToolFollowUp::SystemMessage(content) => {
                    messages.push(message("system", content));
                }
                ToolFollowUp::Stop => stop = true,
            }
        }
        stop
    }
}

fn message(role: &str, content: String) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_follow_ups_are_appended_after_tool_messages() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::UserMessage("User answer".into()));
        outcome.record(ToolFollowUp::SystemMessage("Backend state".into()));
        let mut messages = vec![message("tool", "Receipt".into())];

        assert!(!outcome.apply_follow_ups(&mut messages));
        assert_eq!(
            messages
                .iter()
                .map(|message| message.role.as_str())
                .collect::<Vec<_>>(),
            ["tool", "user", "system"]
        );
    }

    #[test]
    fn stop_follow_up_ends_the_batch_without_fabricating_a_message() {
        let mut outcome = ToolExecutionOutcome::default();
        outcome.record(ToolFollowUp::Stop);
        let mut messages = Vec::new();

        assert!(outcome.apply_follow_ups(&mut messages));
        assert!(messages.is_empty());
    }
}
