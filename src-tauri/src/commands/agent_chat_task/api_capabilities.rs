use super::params::StreamTaskParams;
use crate::services::llm::{self, provider_model_lookup, tool_capable};

pub(super) struct ApiCapabilities {
    pub tools: bool,
    pub thinking: bool,
    pub vision: bool,
}

pub(super) async fn resolve(
    params: &StreamTaskParams,
    canonical_provider: &str,
) -> ApiCapabilities {
    let local = provider_model_lookup::local_capabilities(canonical_provider, &params.model);
    let registered = match local {
        Some(caps) => Some(caps),
        None => provider_model_lookup::capabilities(canonical_provider, &params.model).await,
    };
    let runtime = llm::runtime_models::lookup(canonical_provider, &params.model);
    let is_local = local.is_some();

    ApiCapabilities {
        tools: params.capability_hints.supports_tools.unwrap_or_else(|| {
            capability(
                is_local,
                registered.as_ref().is_some_and(|caps| caps.supports_tools),
                runtime.as_ref().is_some_and(|model| model.supports_tools),
                tool_capable::supports_tools(canonical_provider, &params.model),
            )
        }),
        thinking: params
            .capability_hints
            .supports_thinking
            .unwrap_or_else(|| {
                params.provider == "codex-oauth"
                    || capability(
                        is_local,
                        registered
                            .as_ref()
                            .is_some_and(|caps| caps.supports_thinking),
                        runtime
                            .as_ref()
                            .is_some_and(|model| model.supports_thinking),
                        tool_capable::supports_thinking(canonical_provider, &params.model),
                    )
            }),
        vision: params.capability_hints.supports_vision.unwrap_or_else(|| {
            params.provider == "codex-oauth"
                || capability(
                    is_local,
                    registered.as_ref().is_some_and(|caps| caps.supports_vision),
                    runtime.as_ref().is_some_and(|model| model.supports_vision),
                    tool_capable::supports_vision(canonical_provider, &params.model),
                )
        }),
    }
}

fn capability(is_local: bool, registered: bool, runtime: bool, fallback: bool) -> bool {
    registered || !is_local && (runtime || fallback)
}

#[cfg(test)]
mod tests {
    use super::capability;

    #[test]
    fn local_false_is_authoritative() {
        assert!(!capability(true, false, true, true));
        assert!(capability(true, true, false, false));
        assert!(capability(false, false, true, false));
    }
}
