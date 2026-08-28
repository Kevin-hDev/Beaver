use crate::services::llm::types::ModelInfo;

const FALLBACK_EFFECTIVE_CONTEXT: u32 = 258_400;
const STANDARD_MODES: &[&str] = &["low", "medium", "high", "xhigh"];
const EXTENDED_MODES: &[&str] = &["low", "medium", "high", "xhigh", "max", "ultra"];
const LUNA_MODES: &[&str] = &["low", "medium", "high", "xhigh", "max"];

pub(super) fn models() -> Vec<ModelInfo> {
    [
        (
            "gpt-5.6-sol",
            FALLBACK_EFFECTIVE_CONTEXT,
            EXTENDED_MODES,
            true,
            None,
        ),
        (
            "gpt-5.6-terra",
            FALLBACK_EFFECTIVE_CONTEXT,
            EXTENDED_MODES,
            true,
            None,
        ),
        (
            "gpt-5.6-luna",
            FALLBACK_EFFECTIVE_CONTEXT,
            LUNA_MODES,
            true,
            None,
        ),
        (
            "gpt-5.3-codex-spark",
            128_000,
            STANDARD_MODES,
            false,
            Some("high"),
        ),
        ("gpt-5.5", 258_000, STANDARD_MODES, true, None),
        ("gpt-5.4", 258_000, STANDARD_MODES, true, None),
        ("gpt-5.4-mini", 258_000, STANDARD_MODES, true, None),
        ("gpt-5.4-pro", 258_000, STANDARD_MODES, true, None),
    ]
    .into_iter()
    .map(|(id, context, modes, vision, default_mode)| ModelInfo {
        id: id.to_string(),
        display_name: Some(id.to_string()),
        owned_by: Some("openai".to_string()),
        context_length: Some(context),
        max_output_tokens: None,
        supports_tools: super::supports_tools(id),
        supports_vision: vision,
        supports_thinking: true,
        // Le fallback ne prouve pas l'éligibilité Fast du compte OAuth.
        supports_fast_mode: false,
        reasoning_modes: modes.iter().map(|mode| (*mode).to_string()).collect(),
        default_reasoning_mode: default_mode.map(str::to_string),
        context_usage_includes_reasoning: true,
        is_free: false,
    })
    .collect()
}
