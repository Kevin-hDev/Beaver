use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderModelConfig {
    pub id: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Vrai uniquement lorsqu'une source tarifaire officielle confirme un coût nul.
    #[serde(default)]
    pub is_free: bool,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub default_output_tokens: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    #[serde(default)]
    pub supports_fast_mode: bool,
    #[serde(default)]
    pub reasoning_modes: Vec<String>,
    #[serde(default)]
    pub default_reasoning_mode: Option<String>,
    /// Le fournisseur accepte `enable_thinking` pour activer ou couper ce modèle.
    #[serde(default)]
    pub supports_reasoning_toggle: bool,
    /// Le fournisseur accepte explicitement le rejeu de `reasoning_content`.
    #[serde(default)]
    pub supports_reasoning_replay: bool,
    /// La route exige `tool_stream` pour restituer les appels d'outils en streaming.
    #[serde(default)]
    pub requires_tool_stream: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProviderModelFile {
    pub(super) provider: String,
    pub(super) schema_version: u8,
    pub(super) verified_at: String,
    pub(super) source_urls: Vec<String>,
    #[serde(default)]
    pub(super) inherits_upstream: bool,
    pub(super) models: Vec<ProviderModelConfig>,
}
