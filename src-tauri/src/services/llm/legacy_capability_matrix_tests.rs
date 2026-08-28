use super::{provider_model_lookup, route};
use crate::services::reasoning;

fn resolved_capabilities(provider: &str, model: &str) -> provider_model_lookup::ModelCapabilities {
    let resolved = provider_model_lookup::resolve_local_or_legacy(provider, model).unwrap();
    provider_model_lookup::ModelCapabilities {
        supports_tools: resolved.supports_tools,
        supports_vision: resolved.supports_vision,
        supports_thinking: resolved.supports_thinking,
    }
}

struct ExpectedCapabilities {
    route: &'static str,
    model: &'static str,
    tools: bool,
    vision: bool,
    thinking: bool,
    modes: &'static [&'static str],
}

#[test]
fn legacy_capability_matrix_preserves_registered_and_fallback_behavior() {
    let expected = [
        ExpectedCapabilities {
            route: "google",
            model: "gemini-3.7-flash",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["low", "medium", "high"],
        },
        ExpectedCapabilities {
            route: "mistral",
            model: "mistral-medium-3-5",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["off", "high"],
        },
        ExpectedCapabilities {
            route: "cerebras",
            model: "gemma-4-31b",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["off", "auto"],
        },
        ExpectedCapabilities {
            route: "openrouter",
            model: "google/gemini-3.7-flash",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["low", "medium", "high"],
        },
        ExpectedCapabilities {
            route: "openai",
            model: "gpt-5.6-luna",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["off", "low", "medium", "high", "xhigh", "max"],
        },
        ExpectedCapabilities {
            route: "deepseek",
            model: "deepseek-v4-flash",
            tools: true,
            vision: false,
            thinking: true,
            modes: &["off", "high", "xhigh"],
        },
        ExpectedCapabilities {
            route: "xai",
            model: "grok-4.6",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["low", "medium", "high", "xhigh"],
        },
        ExpectedCapabilities {
            route: "xai-oauth",
            model: "grok-4.6",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["low", "medium", "high", "xhigh"],
        },
        ExpectedCapabilities {
            route: "moonshot",
            model: "kimi-k2.6",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["off", "auto"],
        },
        ExpectedCapabilities {
            route: "moonshot-oauth",
            model: "kimi-k2.6",
            tools: true,
            vision: true,
            thinking: true,
            modes: &["off", "auto"],
        },
        ExpectedCapabilities {
            route: "zai",
            model: "glm-4.5-flash",
            tools: true,
            vision: false,
            thinking: true,
            modes: &["off", "auto"],
        },
    ];

    for item in expected {
        let provider = route::canonical_provider_id(item.route);
        let capabilities = resolved_capabilities(provider, item.model);
        assert_eq!(
            capabilities.supports_tools, item.tools,
            "{}/{}",
            item.route, item.model
        );
        assert_eq!(
            capabilities.supports_vision, item.vision,
            "{}/{}",
            item.route, item.model
        );
        assert_eq!(
            capabilities.supports_thinking, item.thinking,
            "{}/{}",
            item.route, item.model
        );
        let modes = reasoning::supported_modes(item.route, item.model, item.thinking);
        assert_eq!(modes, item.modes, "{}/{}", item.route, item.model);
    }
}

#[test]
fn legacy_capability_matrix_preserves_codex_behavior() {
    assert!(crate::services::codex_client::supports_tools(
        "gpt-5.6-luna"
    ));
    assert!(reasoning::provider_model_supports_thinking(
        "codex-oauth",
        "gpt-5.6-luna"
    ));
    assert_eq!(
        reasoning::supported_modes("codex-oauth", "gpt-5.6-luna", true),
        ["low", "medium", "high", "xhigh", "max"]
    );
}
