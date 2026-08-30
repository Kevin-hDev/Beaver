mod cache_policies;
mod catalog;
mod catalog_api;
mod catalog_local;
mod catalog_oauth;
mod endpoint_types;
mod payload_policies;
mod policies;
mod policy_types;
mod tool_limit_policies;
mod tool_policies;
mod types;
mod wire_contracts;

#[cfg(test)]
pub(super) use catalog::{all, find_id};
pub(super) use catalog::{configurable, find, public_api};
pub(crate) use policy_types::{
    AuthProbePolicy, CachePolicy, ErrorPolicy, ExtensionToolPolicy, ParameterPolicy,
    ResolvedCachePolicy, ResolvedPayloadPolicy, ResolvedToolLimitPolicy, ResolvedToolPolicy,
    SchemaPolicy, ToolLimitPolicy, UpstreamToolFamily,
};
pub(super) use types::*;
pub(crate) use types::{ApiKeyHeader, ImageFormat, MessageWirePolicy, ToolResultPlacement};

pub(crate) fn tool_policy(provider_id: &str, model: &str) -> Option<ResolvedToolPolicy> {
    let profile = find(provider_id)?;
    Some(tool_policies::resolve(profile, model))
}

pub(crate) fn cache_policy<'a>(
    provider_id: &str,
    model: &'a str,
) -> Option<ResolvedCachePolicy<'a>> {
    let profile = find(provider_id)?;
    Some(cache_policies::resolve(profile, model))
}

pub(crate) fn payload_policy(provider_id: &str, model: &str) -> Option<ResolvedPayloadPolicy> {
    let profile = find(provider_id)?;
    Some(payload_policies::resolve(profile, model))
}

pub(crate) fn tool_limit_policy(provider_id: &str, model: &str) -> Option<ResolvedToolLimitPolicy> {
    let profile = find(provider_id)?;
    Some(tool_limit_policies::resolve(profile, model))
}

pub(crate) fn error_policy(provider_id: &str) -> Option<ErrorPolicy> {
    Some(find(provider_id)?.policies.errors)
}

pub(crate) fn has_dynamic_reasoning_catalog(provider_id: &str) -> bool {
    find(provider_id).is_some_and(|profile| profile.policies.dynamic_reasoning_catalog)
}

pub(crate) fn is_local(provider_id: &str) -> bool {
    find(provider_id).is_some_and(|profile| profile.client == ClientSelector::OllamaLocal)
}

pub(crate) fn diagnostic_payload_kind(provider_id: &str) -> Option<&'static str> {
    let family = find(provider_id)?.wire.family;
    Some(match family {
        WireFamily::OpenAiResponses => "responses",
        WireFamily::AnthropicMessages => "anthropic_messages",
        WireFamily::OpenAiChatCompletions | WireFamily::OllamaNative => "chat_completions",
    })
}

pub(crate) fn request_timeout_seconds(provider_id: &str, fallback: u64) -> u64 {
    find(provider_id).map_or(fallback, |profile| {
        if profile.id == crate::services::reasoning_continuity::contract::RouteId::DeepSeek {
            600
        } else {
            fallback
        }
    })
}

pub(crate) fn requires_gemma4_thinking_guard(provider_id: &str, model: &str) -> bool {
    let Some(profile) = find(provider_id) else {
        return false;
    };
    if !profile.policies.gemma4_thinking_guard {
        return false;
    }
    let compact: String = model
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect();
    compact.contains("gemma4")
}

#[cfg(test)]
pub(super) fn anthropic_fixture(
    max_tokens: Option<u32>,
) -> Result<serde_json::Value, &'static str> {
    payload_policies::anthropic_fixture(max_tokens)
}

#[cfg(test)]
mod tests;
