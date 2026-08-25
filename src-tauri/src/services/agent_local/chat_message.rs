use super::types_ollama::{ChatMessage, ToolCallOllama};
use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;

impl ChatMessage {
    pub fn system(content: String) -> Self {
        Self::new("system", content)
    }

    pub fn user(content: String) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(
        content: String,
        display_thinking: Option<String>,
        continuation: Option<ReasoningEnvelope>,
        legacy_tool_loop_reasoning: Option<String>,
        tool_calls: Option<Vec<ToolCallOllama>>,
    ) -> Self {
        Self {
            role: "assistant".to_owned(),
            content,
            images: None,
            tool_calls,
            tool_name: None,
            tool_call_id: None,
            display_thinking,
            continuation,
            legacy_tool_loop_reasoning,
        }
    }

    pub fn tool(content: String, tool_call_id: Option<String>, tool_name: Option<String>) -> Self {
        Self {
            role: "tool".to_owned(),
            content,
            images: None,
            tool_calls: None,
            tool_name,
            tool_call_id,
            display_thinking: None,
            continuation: None,
            legacy_tool_loop_reasoning: None,
        }
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "image-bearing messages currently enter through IPC"
        )
    )]
    pub fn with_images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    fn new(role: &str, content: String) -> Self {
        Self {
            role: role.to_owned(),
            content,
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            display_thinking: None,
            continuation: None,
            legacy_tool_loop_reasoning: None,
        }
    }
}
