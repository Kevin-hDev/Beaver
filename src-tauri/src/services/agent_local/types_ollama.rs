use serde::{Deserialize, Serialize};

pub use super::types_stream::{PullProgress, StreamEvent, StreamOutcome, StreamResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub family: String,
    pub parameter_size: String,
    pub quantization: String,
    pub architecture: String,
    pub is_moe: bool,
    pub context_length: u64,
    pub capabilities: Vec<String>,
    pub digest_short: String,
    pub aliases: Vec<String>,
    pub is_customized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization: String,
    pub architecture: String,
    pub is_moe: bool,
    pub context_length: u64,
    pub capabilities: Vec<String>,
    pub has_audio: bool,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModel {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub is_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModelDetails {
    pub name: String,
    pub description_short: String,
    pub description_long_markdown: String,
    pub capabilities: Vec<String>,
    pub sizes: Vec<String>,
    pub context_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryTag {
    pub name: String,
    pub digest_short: String,
    pub size_gb: Option<f64>,
    pub context_length: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OllamaThink {
    Bool(bool),
    Level(String),
}

impl OllamaThink {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Level(_) => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
    pub think: Option<OllamaThink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(default)]
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallOllama>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Requis par OpenAI-compat pour les messages `role: "tool"` — ignoré par Ollama.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_call_id: Option<String>,
    /// Contenu thinking/reasoning renvoyé au provider pour continuité multi-tour.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallOllama {
    /// ID assigné par les providers OpenAI-compat (ex: "call_abc123"). Absent pour Ollama.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub extra_content: Option<serde_json::Value>,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}
