use super::*;

fn ids(provider_id: &str) -> Vec<String> {
    list(provider_id)
        .iter()
        .map(|model| model.id.clone())
        .collect()
}

#[test]
fn canonical_inventory_sizes_match_the_official_catalogs() {
    for (provider, expected) in [
        ("google", 14),
        ("mistral", 8),
        ("cerebras", 3),
        ("openrouter", 0),
        ("openai", 19),
        ("deepseek", 2),
        ("xai", 7),
        ("moonshot", 15),
        ("zai", 20),
        ("anthropic", 1),
        ("qwen", 76),
    ] {
        assert_eq!(list(provider).len(), expected, "{provider}");
    }
}

#[test]
fn qwen_fallback_exposes_verified_reasoning_contracts() {
    let models = list("qwen");
    assert_eq!(models.len(), 76);
    let flash = &models[0];
    assert_eq!(flash.id, "qwen3.8-flash");
    assert_eq!(flash.context_window, 1_000_000);
    assert_eq!(flash.max_output_tokens, Some(131_072));
    assert!(flash.supports_tools);
    assert!(flash.supports_vision);
    assert!(flash.supports_thinking);
    assert_eq!(flash.reasoning_modes, ["off", "low", "medium", "xhigh"]);
    assert_eq!(flash.default_reasoning_mode.as_deref(), Some("xhigh"));

    let max = &models[1];
    assert_eq!(max.id, "qwen3.8-max");
    assert_eq!(max.context_window, 1_000_000);
    assert_eq!(max.max_output_tokens, Some(131_072));
    assert!(max.supports_tools);
    assert!(max.supports_vision);
    assert!(max.supports_thinking);
    assert_eq!(max.reasoning_modes, ["off", "low", "medium", "xhigh"]);
    assert_eq!(max.default_reasoning_mode.as_deref(), Some("xhigh"));

    let plus = lookup("qwen", "qwen3.7-plus-2026-05-26").unwrap();
    assert_eq!(plus.reasoning_modes, ["off", "auto"]);
    assert_eq!(plus.default_reasoning_mode.as_deref(), Some("auto"));
    assert!(plus.supports_vision);

    let thinking_only = lookup("qwen", "qwen3.7-max-preview").unwrap();
    assert_eq!(thinking_only.reasoning_modes, ["auto"]);
    assert_eq!(
        thinking_only.default_reasoning_mode.as_deref(),
        Some("auto")
    );
}

#[test]
fn model_studio_inventory_covers_every_documented_thinking_family() {
    for (model, expected_modes) in [
        ("qwen3.5-plus", &["off", "auto"][..]),
        ("qwen3-vl-plus", &["off", "auto"][..]),
        ("qwen3-vl-235b-a22b-thinking", &["auto"][..]),
        ("deepseek-v4-pro", &["off", "high", "max"][..]),
        ("deepseek-r1", &["auto"][..]),
        (
            "glm-5.2",
            &["off", "low", "medium", "high", "xhigh", "max"][..],
        ),
        ("ZHIPU/GLM-5.3", &["low", "high", "max"][..]),
        ("kimi-k2.6", &["off", "auto"][..]),
        ("kimi-k2.7-code", &["auto"][..]),
        ("MiniMax-M2.5", &["auto"][..]),
    ] {
        let entry = lookup("qwen", model).unwrap_or_else(|| panic!("missing qwen/{model}"));
        assert!(entry.supports_thinking, "{model}");
        assert_eq!(entry.reasoning_modes, expected_modes, "{model}");
    }
}

#[test]
fn model_studio_keeps_tool_transport_and_distilled_limits_exact() {
    for model in ["glm-5.2", "glm-5.1", "glm-5", "glm-4.7", "glm-4.6"] {
        let entry = lookup("qwen", model).unwrap();
        assert!(entry.supports_tools, "{model}");
        assert!(entry.requires_tool_stream, "{model}");
    }
    assert!(!lookup("qwen", "glm-4.5").unwrap().requires_tool_stream);

    for model in [
        "deepseek-r1-distill-qwen-32b",
        "deepseek-r1-distill-qwen-14b",
        "deepseek-r1-distill-qwen-7b",
        "deepseek-r1-distill-qwen-1.5b",
        "deepseek-r1-distill-llama-70b",
        "deepseek-r1-distill-llama-8b",
    ] {
        let entry = lookup("qwen", model).unwrap();
        assert!(entry.supports_thinking, "{model}");
        assert!(!entry.supports_tools, "{model}");
        assert_eq!(entry.max_output_tokens, Some(16_384), "{model}");
    }

    assert_eq!(
        lookup("qwen", "deepseek-v4-pro-0813")
            .unwrap()
            .max_output_tokens,
        Some(393_216)
    );
    assert_eq!(
        lookup("qwen", "deepseek-v4-pro-us").unwrap().id,
        "deepseek-v4-pro"
    );
}

#[test]
fn anthropic_fallback_exposes_only_the_validated_haiku_model() {
    let models = list("anthropic");
    assert_eq!(models.len(), 1);
    let haiku = &models[0];
    assert_eq!(haiku.id, "claude-haiku-4-5-20251001");
    assert_eq!(haiku.context_window, 200_000);
    assert_eq!(haiku.max_output_tokens, Some(64_000));
    assert!(haiku.supports_tools);
    assert!(haiku.supports_vision);
    assert!(haiku.supports_thinking);
    assert_eq!(haiku.reasoning_modes, ["off", "low", "medium", "high"]);
    assert_eq!(haiku.default_reasoning_mode.as_deref(), Some("medium"));
}

#[test]
fn removed_groq_provider_has_no_embedded_models() {
    assert!(list("groq").is_empty());
}

#[test]
fn inventories_keep_current_canonical_ids() {
    assert_eq!(
        ids("mistral"),
        [
            "mistral-medium-3-5",
            "mistral-small-2603",
            "mistral-large-2512",
            "codestral-2508",
            "labs-leanstral-1-5",
            "ministral-14b-2512",
            "ministral-8b-2512",
            "ministral-3b-2512",
        ]
    );
    assert_eq!(
        ids("cerebras"),
        ["zai-glm-4.7", "gemma-4-31b", "gpt-oss-120b"]
    );
    assert_eq!(
        ids("xai"),
        [
            "grok-4.6",
            "grok-4.5",
            "grok-build-0.1",
            "grok-4.3",
            "grok-4.20-multi-agent-0309",
            "grok-4.20-0309-reasoning",
            "grok-4.20-0309-non-reasoning",
        ]
    );
}

#[test]
fn retired_or_invented_ids_are_not_local_models() {
    for (provider, model) in [
        ("mistral", "mistral-medium-3.5"),
        ("mistral", "mistral-small-4"),
        ("mistral", "devstral-small-latest"),
        ("mistral", "magistral-medium-latest"),
        ("openai", "gpt-5.6-terra-pro"),
        ("cerebras", "qwen-3-235b-a22b-instruct-2507"),
        ("cerebras", "llama3.1-8b"),
        ("moonshot", "kimi-latest"),
        ("moonshot", "kimi-k2-thinking"),
        ("zai", "glm-5-code"),
    ] {
        assert!(lookup(provider, model).is_none(), "{provider}/{model}");
    }
}

#[test]
fn corrected_limits_and_capabilities_are_stable() {
    let gemini = lookup("google", "gemini-2.5-flash-lite").unwrap();
    assert_eq!(gemini.context_window, 1_048_576);
    assert_eq!(gemini.max_output_tokens, Some(65_536));

    let cerebras = lookup("cerebras", "gemma-4-31b").unwrap();
    assert_eq!(cerebras.max_output_tokens, Some(40_960));
    assert!(cerebras.supports_vision);

    let openai_mini = lookup("openai", "gpt-5.4-mini").unwrap();
    assert_eq!(openai_mini.context_window, 400_000);
    assert_eq!(openai_mini.max_output_tokens, Some(128_000));

    let kimi = lookup("moonshot", "kimi-k3").unwrap();
    assert_eq!(kimi.context_window, 1_048_576);
    assert_eq!(kimi.max_output_tokens, Some(1_048_576));
    assert_eq!(kimi.default_output_tokens, Some(131_072));

    let glm_vision = lookup("zai", "glm-4.5v").unwrap();
    assert_eq!(glm_vision.context_window, 64_000);
    assert_eq!(glm_vision.max_output_tokens, Some(16_384));
    assert!(glm_vision.supports_vision);
}

#[test]
fn openai_fast_mode_is_limited_to_the_verified_api_models() {
    for model in ["gpt-5.6-sol", "gpt-5.6", "gpt-5.6-terra", "gpt-5.6-luna"] {
        let entry = lookup("openai", model).unwrap();
        assert!(entry.supports_fast_mode, "{model}");
        assert_eq!(
            entry.reasoning_modes,
            ["off", "low", "medium", "high", "xhigh", "max"],
            "{model} must keep the reasoning menu that hosts the Fast toggle"
        );
    }
    assert!(lookup("openai", "gpt-5.6-terra-pro").is_none());
    for model in ["gpt-5.5", "gpt-5.4", "gpt-5.4-mini"] {
        assert!(
            !crate::services::llm::provider_model_lookup::supports_fast_mode("openai", model),
            "{model} must remain outside the closed Fast inventory"
        );
    }
    assert!(
        !crate::services::llm::provider_model_lookup::supports_fast_mode(
            "openai",
            "openai/gpt-5.6-sol",
        )
    );
    assert!(
        !crate::services::llm::provider_model_lookup::supports_fast_mode(
            "openrouter",
            "openai/gpt-5.6-sol",
        )
    );
}

#[test]
fn snapshot_models_do_not_need_the_legacy_name_fallback() {
    let mut gaps = Vec::new();
    for source in SOURCES {
        for model in list(source.provider_id) {
            let resolved = crate::services::llm::provider_model_lookup::resolve_local(
                source.provider_id,
                &model.id,
            )
            .unwrap();
            if resolved.provenance
                != crate::services::llm::provider_model_lookup::CapabilityProvenance::EmbeddedRegistry
            {
                gaps.push(format!("{}/{}", source.provider_id, model.id));
            }
        }
    }
    assert!(gaps.is_empty(), "missing explicit capabilities: {gaps:?}");
}
