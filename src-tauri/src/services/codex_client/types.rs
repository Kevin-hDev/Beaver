use serde::Serialize;

pub const CODEX_API_BASE: &str = "https://chatgpt.com/backend-api/codex";

#[derive(Serialize)]
pub struct CodexRequest {
    pub model: String,
    pub instructions: String,
    pub input: Vec<serde_json::Value>,
    pub stream: bool,
    pub store: bool,
    pub tools: Vec<serde_json::Value>,
    pub tool_choice: String,
    pub parallel_tool_calls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    pub include: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ReasoningConfig {
    pub effort: String,
    pub summary: String,
}
