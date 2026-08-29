use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageApiFormat {
    #[default]
    ChatCompletions,
    Responses,
    GeminiNative,
    AnthropicMessages,
}

#[derive(Debug, Clone, Copy)]
pub struct UsageContext<'a> {
    pub canonical_provider_id: &'a str,
    pub model: &'a str,
    pub api_format: UsageApiFormat,
}

impl<'a> UsageContext<'a> {
    pub const fn chat(canonical_provider_id: &'a str, model: &'a str) -> Self {
        Self {
            canonical_provider_id,
            model,
            api_format: UsageApiFormat::ChatCompletions,
        }
    }

    pub const fn responses(canonical_provider_id: &'a str, model: &'a str) -> Self {
        Self {
            canonical_provider_id,
            model,
            api_format: UsageApiFormat::Responses,
        }
    }
}
