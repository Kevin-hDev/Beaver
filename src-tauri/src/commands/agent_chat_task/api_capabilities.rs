use super::params::StreamCapabilityHints;
use crate::services::llm::provider_model_lookup;

pub(crate) struct ApiCapabilities {
    pub tools: bool,
    pub thinking: bool,
    pub vision: bool,
}

pub(crate) async fn resolve(
    provider: &str,
    model: &str,
    _hints: &StreamCapabilityHints,
) -> ApiCapabilities {
    let capability_provider = crate::services::llm::route::canonical_provider_id(provider);
    let resolved: Option<provider_model_lookup::ResolvedModelCapabilities> =
        provider_model_lookup::resolve(capability_provider, model).await;

    ApiCapabilities {
        tools: resolved.as_ref().is_some_and(|caps| caps.supports_tools),
        thinking: resolved.as_ref().is_some_and(|caps| caps.supports_thinking),
        vision: resolved.as_ref().is_some_and(|caps| caps.supports_vision),
    }
}

#[cfg(test)]
#[path = "api_capabilities_tests.rs"]
mod tests;
