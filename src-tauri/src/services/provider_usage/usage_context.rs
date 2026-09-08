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
    pub(super) fn cache_counts_fit_input(
        self,
        input: u64,
        read: Option<u64>,
        write: Option<u64>,
    ) -> bool {
        // Gemini explicit caching creates a resource, then reads that same
        // prefix for generation. Both operations can count the same tokens.
        read.is_none_or(|count| count <= input)
            && write.is_none_or(|count| count <= input)
            && (self.cache_writes_overlap_reads()
                || read.unwrap_or(0).saturating_add(write.unwrap_or(0)) <= input)
    }

    pub(super) fn cache_writes_overlap_reads(self) -> bool {
        self.api_format == UsageApiFormat::ChatCompletions
            && crate::services::llm::route_profile::cache_policy(
                self.canonical_provider_id,
                self.model,
            )
            .is_some_and(|policy| {
                policy.kind == crate::services::llm::route_profile::CachePolicy::OpenRouterGemini
            })
    }

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
