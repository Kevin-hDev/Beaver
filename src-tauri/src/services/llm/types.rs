//! Types communs du module LLM multi-provider.

use serde::{Deserialize, Serialize};

use super::model_reasoning_contract::ModelReasoningContract;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<Vec<String>>,
    // Parsing provenance, not a second user-facing capability contract.
    #[serde(skip)]
    pub(crate) catalog_capabilities: super::openrouter_model_metadata::CatalogCapabilities,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_contract: Option<ModelReasoningContract>,
    #[serde(default)]
    pub supports_fast_mode: bool,
    #[serde(default = "default_true")]
    pub context_usage_includes_reasoning: bool,
    #[serde(default)]
    pub is_free: bool,
}

#[cfg(test)]
impl ModelInfo {
    pub fn reasoning_modes(&self) -> Vec<String> {
        self.reasoning_contract
            .as_ref()
            .map(ModelReasoningContract::selection)
            .map(|(modes, _)| modes)
            .unwrap_or_default()
    }

    pub fn default_reasoning_mode(&self) -> Option<String> {
        self.reasoning_contract
            .as_ref()
            .and_then(|contract| contract.selection().1)
    }
}

const fn default_true() -> bool {
    true
}

/// Erreurs du module LLM. Volontairement basées sur `String` pour cohérence
/// avec le reste du projet (les commandes Tauri retournent `Result<_, String>`).
#[derive(Debug, Clone)]
pub enum LlmError {
    Unauthorized,
    RateLimit { retry_after_secs: Option<u64> },
    KnownProvider(super::provider_error::ProviderErrorCode),
    Http { status: u16, message: String },
    Network(String),
    Parse(String),
    Provider(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Unauthorized => write!(f, "clé API invalide ou non autorisée"),
            LlmError::RateLimit { retry_after_secs } => match retry_after_secs {
                Some(s) => write!(f, "rate limit atteint, réessaie dans {}s", s),
                None => write!(f, "rate limit atteint, réessaie plus tard"),
            },
            LlmError::KnownProvider(code) => f.write_str(code.as_str()),
            LlmError::Http { status, message } => write!(f, "HTTP {}: {}", status, message),
            LlmError::Network(_) => write!(f, "erreur réseau — vérifiez votre connexion"),
            LlmError::Parse(m) => write!(f, "erreur de parsing : {}", m),
            LlmError::Provider(m) => write!(f, "erreur provider : {}", m),
        }
    }
}

impl From<LlmError> for String {
    fn from(e: LlmError) -> Self {
        e.to_string()
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
