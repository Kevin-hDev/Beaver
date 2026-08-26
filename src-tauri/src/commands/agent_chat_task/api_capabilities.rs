use super::params::StreamCapabilityHints;
use crate::services::llm::{self, provider_model_lookup, tool_capable};

pub(super) struct ApiCapabilities {
    pub tools: bool,
    pub thinking: bool,
    pub vision: bool,
}

pub(super) async fn resolve(
    provider: &str,
    model: &str,
    _hints: &StreamCapabilityHints,
) -> ApiCapabilities {
    let capability_provider = crate::services::llm::route::canonical_provider_id(provider);
    let local = provider_model_lookup::local_capabilities(capability_provider, model);
    let registered = match local {
        Some(caps) => Some(caps),
        None => provider_model_lookup::capabilities(capability_provider, model).await,
    };
    let runtime = llm::runtime_models::lookup(capability_provider, model);
    let is_local = local.is_some();

    ApiCapabilities {
        tools: tools_capability(provider, model, {
            capability(
                is_local,
                registered.as_ref().is_some_and(|caps| caps.supports_tools),
                runtime.as_ref().is_some_and(|model| model.supports_tools),
                tool_capable::supports_tools(capability_provider, model),
            )
        }),
        thinking: provider == crate::services::codex_client::PROVIDER_ID
            || capability(
                is_local,
                registered
                    .as_ref()
                    .is_some_and(|caps| caps.supports_thinking),
                runtime
                    .as_ref()
                    .is_some_and(|model| model.supports_thinking),
                tool_capable::supports_thinking(capability_provider, model),
            ),
        vision: provider == crate::services::codex_client::PROVIDER_ID
            || capability(
                is_local,
                registered.as_ref().is_some_and(|caps| caps.supports_vision),
                runtime.as_ref().is_some_and(|model| model.supports_vision),
                tool_capable::supports_vision(capability_provider, model),
            ),
    }
}

fn tools_capability(provider: &str, model: &str, detected: bool) -> bool {
    if provider == crate::services::codex_client::PROVIDER_ID {
        crate::services::codex_client::supports_tools(model)
    } else {
        detected
    }
}

fn capability(is_local: bool, registered: bool, runtime: bool, fallback: bool) -> bool {
    registered || !is_local && (runtime || fallback)
}

#[cfg(test)]
#[path = "api_capabilities_tests.rs"]
mod tests;
