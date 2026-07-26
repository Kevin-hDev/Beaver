use serde::Serialize;

pub const CODEX_API_BASE: &str = "https://chatgpt.com/backend-api/codex";

pub struct CodexModelSpec {
    pub id: &'static str,
    pub context_length: u32,
    pub reasoning_modes: &'static [&'static str],
    pub supports_vision: bool,
}

const STANDARD_REASONING_MODES: &[&str] = &["low", "medium", "high", "xhigh"];

pub const CODEX_MODELS: &[CodexModelSpec] = &[
    CodexModelSpec {
        id: "gpt-5.6-sol",
        context_length: 372_000,
        reasoning_modes: &["low", "medium", "high", "xhigh", "max", "ultra"],
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.6-terra",
        context_length: 372_000,
        reasoning_modes: &["low", "medium", "high", "xhigh", "max", "ultra"],
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.6-luna",
        context_length: 372_000,
        reasoning_modes: &["low", "medium", "high", "xhigh", "max"],
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.3-codex-spark",
        context_length: 128_000,
        reasoning_modes: STANDARD_REASONING_MODES,
        supports_vision: false,
    },
    CodexModelSpec {
        id: "gpt-5.5",
        context_length: 258_000,
        reasoning_modes: STANDARD_REASONING_MODES,
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.4",
        context_length: 258_000,
        reasoning_modes: STANDARD_REASONING_MODES,
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.4-mini",
        context_length: 258_000,
        reasoning_modes: STANDARD_REASONING_MODES,
        supports_vision: true,
    },
    CodexModelSpec {
        id: "gpt-5.4-pro",
        context_length: 258_000,
        reasoning_modes: STANDARD_REASONING_MODES,
        supports_vision: true,
    },
];

#[derive(Serialize)]
pub struct CodexRequest {
    pub model: String,
    pub instructions: String,
    pub input: Vec<serde_json::Value>,
    pub stream: bool,
    pub store: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct ReasoningConfig {
    pub effort: String,
    pub summary: String,
}
