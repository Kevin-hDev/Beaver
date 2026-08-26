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
                    append_to_tool(messages, "User follow-up", &content)?;
                }
                ToolFollowUp::SystemMessage(content) => {
                    append_to_tool(messages, "System follow-up", &content)?;
                }
                ToolFollowUp::Stop => stop = true,
            }
        }
        Ok(stop)
    }
}

fn append_to_tool(messages: &mut [ChatMessage], label: &str, content: &str) -> Result<(), String> {
    let tool = messages
        .iter_mut()
        .rev()
        .find(|message| message.role == "tool")
        .ok_or_else(generic_error)?;
    if content.is_empty()
        || content.len() > MAX_FOLLOW_UP_BYTES
        || tool.content.len().saturating_add(content.len()) > MAX_FOLLOW_UP_BYTES
    {
        return Err(generic_error());
    }
    tool.content.push_str("\n\n");
    tool.content.push_str(label);
    tool.content.push_str(":\n");
    tool.content.push_str(content);
    Ok(())
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
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
}
