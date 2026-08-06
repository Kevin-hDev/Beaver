use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptMode {
    Chatbot,
    Agentic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptTier {
    Compact,
    Detailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "content", rename_all = "lowercase")]
pub enum PromptOverride {
    Custom(String),
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptSelection {
    Default,
    Beaver,
    Custom,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptSource {
    Beaver,
    Ollama,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPromptView {
    pub content: String,
    pub source: PromptSource,
    pub selection: PromptSelection,
    pub disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_prompt_available: Option<bool>,
}
