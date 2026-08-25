use super::types_ollama::{ChatMessage, ToolCallOllama};

impl ChatMessage {
    pub fn system(content: String) -> Self {
        Self::new("system", content)
    }

    pub fn user(content: String) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(
        content: String,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCallOllama>>,
    ) -> Self {
        Self {
            role: "assistant".to_owned(),
            content,
            images: None,
            tool_calls,
            tool_name: None,
            tool_call_id: None,
            reasoning_content,
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
            reasoning_content: None,
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
            reasoning_content: None,
        }
    }
}
