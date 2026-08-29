use serde::{Deserialize, Serialize};

use super::limits::{validate_credential_scope, validate_model_id, LimitError};

pub use super::continuation_target::{ContinuationTarget, NonReplayTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractId {
    OllamaNativeV1,
    GeminiCompatV1,
    MistralChunksV1,
    CerebrasChatV1,
    #[serde(rename = "openrouter-details-v1")]
    OpenRouterDetailsV1,
    #[serde(rename = "openai-responses-v1")]
    OpenAiResponsesV1,
    #[serde(rename = "deepseek-chat-v1")]
    DeepSeekChatV1,
    XaiResponsesV1,
    KimiChatV1,
    ZaiChatV1,
    CodexResponsesV1,
    AnthropicMessagesV1,
}

impl ContractId {
    #[cfg(test)]
    pub const ALL: [Self; 12] = [
        Self::OllamaNativeV1,
        Self::GeminiCompatV1,
        Self::MistralChunksV1,
        Self::CerebrasChatV1,
        Self::OpenRouterDetailsV1,
        Self::OpenAiResponsesV1,
        Self::DeepSeekChatV1,
        Self::XaiResponsesV1,
        Self::KimiChatV1,
        Self::ZaiChatV1,
        Self::CodexResponsesV1,
        Self::AnthropicMessagesV1,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteId {
    Ollama,
    Google,
    Mistral,
    Cerebras,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "deepseek")]
    DeepSeek,
    Xai,
    XaiOauth,
    Moonshot,
    MoonshotOauth,
    Zai,
    CodexOauth,
    Anthropic,
    Qwen,
}

impl RouteId {
    pub const ALL: [Self; 15] = [
        Self::Ollama,
        Self::Google,
        Self::Mistral,
        Self::Cerebras,
        Self::OpenRouter,
        Self::OpenAi,
        Self::DeepSeek,
        Self::Xai,
        Self::XaiOauth,
        Self::Moonshot,
        Self::MoonshotOauth,
        Self::Zai,
        Self::CodexOauth,
        Self::Anthropic,
        Self::Qwen,
    ];

    pub fn from_provider_id(provider: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|route| route.provider_id() == provider)
    }

    pub const fn provider_id(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Google => "google",
            Self::Mistral => "mistral",
            Self::Cerebras => "cerebras",
            Self::OpenRouter => "openrouter",
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::Xai => "xai",
            Self::XaiOauth => "xai-oauth",
            Self::Moonshot => "moonshot",
            Self::MoonshotOauth => "moonshot-oauth",
            Self::Zai => "zai",
            Self::CodexOauth => "codex-oauth",
            Self::Anthropic => "anthropic",
            Self::Qwen => "qwen",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningModeId {
    Off,
    Auto,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
    Ultra,
}

impl ReasoningModeId {
    pub fn from_name(mode: Option<&str>) -> Option<Self> {
        match mode.unwrap_or("off") {
            "off" => Some(Self::Off),
            "auto" => Some(Self::Auto),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" => Some(Self::Xhigh),
            "max" => Some(Self::Max),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationUse {
    UserContinuation,
    ToolContinuation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct CredentialScope(String);

impl CredentialScope {
    pub const LOCAL_UNCREDENTIALED: &'static str = "local-uncredentialed";

    pub fn authenticated(value: impl Into<String>) -> Result<Self, LimitError> {
        let value = value.into();
        validate_credential_scope(&value)?;
        if value == Self::LOCAL_UNCREDENTIALED {
            return Err(LimitError::CredentialScope);
        }
        Ok(Self(value))
    }

    pub fn local_uncredentialed() -> Self {
        Self(Self::LOCAL_UNCREDENTIALED.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate_for_route(&self, route_id: RouteId) -> Result<(), LimitError> {
        let is_local = self.0 == Self::LOCAL_UNCREDENTIALED;
        let expects_local = route_id == RouteId::Ollama;
        (is_local == expects_local)
            .then_some(())
            .ok_or(LimitError::CredentialScope)
    }
}

impl<'de> Deserialize<'de> for CredentialScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        validate_credential_scope(&value).map_err(serde::de::Error::custom)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTarget {
    pub route_id: RouteId,
    pub model_id: String,
    pub credential_scope: CredentialScope,
    pub reasoning_mode: ReasoningModeId,
    pub continuation_use: ContinuationUse,
}

impl ReplayTarget {
    pub fn validate(&self) -> Result<(), LimitError> {
        validate_model_id(&self.model_id)?;
        validate_credential_scope(self.credential_scope.as_str())?;
        self.credential_scope.validate_for_route(self.route_id)
    }
}
