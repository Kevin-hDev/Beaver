use super::system_prompt_resolver::{resolve_global, resolve_ollama};
use super::system_prompt_store::SystemPromptSettings;
use super::system_prompt_types::{
    PromptMode, PromptOverride, PromptSource, PromptTier,
};

#[test]
fn empty_custom_prompt_is_preserved_as_an_explicit_disabled_state() {
    let mut settings = SystemPromptSettings::default();

    settings
        .set_global(PromptMode::Agentic, PromptTier::Compact, "")
        .unwrap();

    assert_eq!(
        settings.global_override(PromptMode::Agentic, PromptTier::Compact),
        Some(&PromptOverride::Disabled)
    );
}

#[test]
fn restoring_one_variant_does_not_change_the_other_variants() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Agentic, PromptTier::Compact, "agent compact")
        .unwrap();
    settings
        .set_global(PromptMode::Agentic, PromptTier::Detailed, "agent detailed")
        .unwrap();

    settings.restore_global(PromptMode::Agentic, PromptTier::Compact);

    assert_eq!(
        settings.global_override(PromptMode::Agentic, PromptTier::Compact),
        None
    );
    assert_eq!(
        settings.global_override(PromptMode::Agentic, PromptTier::Detailed),
        Some(&PromptOverride::Custom("agent detailed".into()))
    );
}

#[test]
fn removing_an_ollama_model_clears_all_its_variants() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_ollama(
            "gemma4:e2b",
            PromptMode::Chatbot,
            PromptTier::Compact,
            "custom",
        )
        .unwrap();

    settings.remove_ollama_model("gemma4:e2b");

    assert_eq!(
        settings.ollama_override("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact),
        None
    );
}

#[test]
fn ollama_custom_prompt_has_priority_over_native_and_global_prompts() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "global")
        .unwrap();
    settings
        .set_ollama(
            "phi4:latest",
            PromptMode::Chatbot,
            PromptTier::Compact,
            "local",
        )
        .unwrap();

    let view = resolve_ollama(
        &settings,
        "phi4:latest",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("native"),
        "beaver",
    );

    assert_eq!(view.content, "local");
    assert_eq!(view.source, PromptSource::Custom);
    assert!(view.customized);
    assert!(!view.disabled);
}

#[test]
fn restoring_an_ollama_prompt_explicitly_returns_to_beaver() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "global")
        .unwrap();
    settings
        .set_ollama(
            "gemma4:e2b",
            PromptMode::Chatbot,
            PromptTier::Compact,
            "custom",
        )
        .unwrap();

    settings.restore_ollama("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact);
    let view = resolve_ollama(
        &settings,
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("Test system prompt"),
        "beaver",
    );

    assert_eq!(view.content, "beaver");
    assert_eq!(view.source, PromptSource::Beaver);
    assert!(!view.customized);
}

#[test]
fn native_ollama_prompt_has_priority_over_the_global_prompt() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Chatbot, PromptTier::Detailed, "global")
        .unwrap();

    let view = resolve_ollama(
        &settings,
        "phi4:latest",
        PromptMode::Chatbot,
        PromptTier::Detailed,
        Some("native"),
        "beaver",
    );

    assert_eq!(view.content, "native");
    assert_eq!(view.source, PromptSource::Ollama);
    assert!(!view.customized);
}

#[test]
fn global_custom_prompt_is_used_when_ollama_has_no_own_prompt() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Agentic, PromptTier::Detailed, "global")
        .unwrap();

    let view = resolve_ollama(
        &settings,
        "qwen:latest",
        PromptMode::Agentic,
        PromptTier::Detailed,
        None,
        "beaver",
    );

    assert_eq!(view.content, "global");
    assert_eq!(view.source, PromptSource::Custom);
    assert!(!view.customized);
}

#[test]
fn explicit_empty_ollama_prompt_blocks_every_inherited_prompt() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Agentic, PromptTier::Compact, "global")
        .unwrap();
    settings
        .set_ollama(
            "gemma4:e2b",
            PromptMode::Agentic,
            PromptTier::Compact,
            "",
        )
        .unwrap();

    let view = resolve_ollama(
        &settings,
        "gemma4:e2b",
        PromptMode::Agentic,
        PromptTier::Compact,
        Some("stale native"),
        "beaver",
    );

    assert_eq!(view.content, "");
    assert_eq!(view.source, PromptSource::Custom);
    assert!(view.customized);
    assert!(view.disabled);
}

#[test]
fn global_empty_prompt_is_custom_and_can_be_restored_to_beaver() {
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "")
        .unwrap();

    let disabled = resolve_global(
        &settings,
        PromptMode::Chatbot,
        PromptTier::Compact,
        "beaver",
    );
    assert_eq!(disabled.content, "");
    assert_eq!(disabled.source, PromptSource::Custom);
    assert!(disabled.customized);
    assert!(disabled.disabled);

    settings.restore_global(PromptMode::Chatbot, PromptTier::Compact);
    let restored = resolve_global(
        &settings,
        PromptMode::Chatbot,
        PromptTier::Compact,
        "beaver",
    );
    assert_eq!(restored.content, "beaver");
    assert_eq!(restored.source, PromptSource::Beaver);
    assert!(!restored.customized);
}

#[test]
fn settings_round_trip_keeps_disabled_and_custom_variants() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let mut settings = SystemPromptSettings::default();
    settings
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "")
        .unwrap();
    settings
        .set_ollama(
            "qwen3:32b",
            PromptMode::Agentic,
            PromptTier::Detailed,
            "local detailed",
        )
        .unwrap();
    settings
        .set_ollama(
            "gemma4:e2b",
            PromptMode::Chatbot,
            PromptTier::Compact,
            "custom",
        )
        .unwrap();
    settings.restore_ollama("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact);

    settings.write_to_path(&path).unwrap();
    let loaded = SystemPromptSettings::read_from_path(&path);

    assert_eq!(
        loaded.global_override(PromptMode::Chatbot, PromptTier::Compact),
        Some(&PromptOverride::Disabled)
    );
    assert_eq!(
        loaded.ollama_override("qwen3:32b", PromptMode::Agentic, PromptTier::Detailed),
        Some(&PromptOverride::Custom("local detailed".into()))
    );
    assert_eq!(
        loaded.ollama_override("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact),
        Some(&PromptOverride::Beaver)
    );
}

#[test]
fn legacy_model_prompt_is_migrated_to_every_mode_and_tier() {
    let directory = tempfile::tempdir().unwrap();
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(
        &legacy_path,
        r#"{"prompts":{"gemma4:e2b":"legacy prompt"}}"#,
    )
    .unwrap();

    let settings = SystemPromptSettings::read_with_legacy(
        &directory.path().join("system-prompt-settings.json"),
        &legacy_path,
    );

    for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
        for tier in [PromptTier::Compact, PromptTier::Detailed] {
            assert_eq!(
                settings.ollama_override("gemma4:e2b", mode, tier),
                Some(&PromptOverride::Custom("legacy prompt".into()))
            );
        }
    }
}
