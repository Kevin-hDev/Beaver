use super::system_prompt_resolver::{
    resolve_global, resolve_ollama, resolve_ollama_native, resolve_ollama_without_native,
};
use super::ollama_native_prompts::NativePromptLookup;
use super::system_prompt_store::{SystemPromptSettings, SystemPromptSettingsStore};
use super::system_prompt_types::{
    PromptMode, PromptOverride, PromptSelection, PromptSource, PromptTier,
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
    assert_eq!(view.selection, PromptSelection::Custom);
    assert!(!view.disabled);
}

#[test]
fn ollama_can_switch_from_native_to_beaver_and_back_to_default() {
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

    settings
        .select_ollama_beaver("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact)
        .unwrap();
    let beaver_view = resolve_ollama(
        &settings,
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("Test system prompt"),
        "beaver",
    );

    assert_eq!(beaver_view.content, "beaver");
    assert_eq!(beaver_view.source, PromptSource::Beaver);
    assert_eq!(beaver_view.selection, PromptSelection::Beaver);

    settings.restore_ollama_default(
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
    )
    .unwrap();
    let native_view = resolve_ollama(
        &settings,
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("native"),
        "beaver",
    );

    assert_eq!(native_view.content, "native");
    assert_eq!(native_view.source, PromptSource::Ollama);
    assert_eq!(native_view.selection, PromptSelection::Default);
}

#[test]
fn explicit_ollama_settings_resolve_without_reading_the_native_prompt() {
    let mut settings = SystemPromptSettings::default();
    settings
        .select_ollama_beaver("gemma4:e2b", PromptMode::Agentic, PromptTier::Compact)
        .unwrap();

    let view = resolve_ollama_without_native(
        &settings,
        "gemma4:e2b",
        PromptMode::Agentic,
        PromptTier::Compact,
        "beaver",
    )
    .expect("an explicit Beaver selection must resolve locally");

    assert_eq!(view.content, "beaver");
    assert_eq!(view.selection, PromptSelection::Beaver);
    assert!(serde_json::to_value(&view)
        .unwrap()
        .get("nativePromptAvailable")
        .is_none());
    assert!(resolve_ollama_without_native(
        &SystemPromptSettings::default(),
        "gemma4:e2b",
        PromptMode::Agentic,
        PromptTier::Compact,
        "beaver",
    )
    .is_none());
}

#[test]
fn selecting_beaver_respects_the_model_collection_limit() {
    let mut settings = SystemPromptSettings::default();
    for index in 0..512 {
        settings
            .set_ollama(
                &format!("model-{index}"),
                PromptMode::Chatbot,
                PromptTier::Compact,
                "custom",
            )
            .unwrap();
    }

    assert!(settings
        .select_ollama_beaver(
            "model-over-limit",
            PromptMode::Chatbot,
            PromptTier::Compact,
        )
        .is_err());
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
    assert_eq!(view.selection, PromptSelection::Default);
    assert_eq!(view.native_prompt_available, Some(true));
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
    assert_eq!(view.selection, PromptSelection::Default);
    assert_eq!(view.native_prompt_available, Some(false));
}

#[test]
fn unknown_native_prompt_availability_is_not_reported_as_absent() {
    let settings = SystemPromptSettings::default();

    let view = resolve_ollama_native(
        &settings,
        "legacy:latest",
        PromptMode::Agentic,
        PromptTier::Compact,
        &NativePromptLookup::Unknown,
        "beaver",
    );

    assert_eq!(view.content, "beaver");
    assert_eq!(view.native_prompt_available, None);
    assert!(serde_json::to_value(&view)
        .unwrap()
        .get("nativePromptAvailable")
        .is_none());
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
    assert_eq!(view.selection, PromptSelection::Disabled);
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
    assert_eq!(disabled.selection, PromptSelection::Disabled);
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
    assert_eq!(restored.selection, PromptSelection::Default);
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
    settings
        .select_ollama_beaver("gemma4:e2b", PromptMode::Chatbot, PromptTier::Compact)
        .unwrap();

    settings.write_to_path(&path).unwrap();
    let loaded = SystemPromptSettings::read_from_path(&path).unwrap();

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
        None
    );
    let restored = resolve_ollama(
        &loaded,
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("native"),
        "beaver",
    );
    assert_eq!(restored.selection, PromptSelection::Beaver);
    assert_eq!(restored.content, "beaver");

    let serialized = std::fs::read_to_string(&path).unwrap();
    assert!(!serialized.contains(r#""state": "beaver""#));
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
    )
    .unwrap();

    for mode in [PromptMode::Chatbot, PromptMode::Agentic] {
        for tier in [PromptTier::Compact, PromptTier::Detailed] {
            assert_eq!(
                settings.ollama_override("gemma4:e2b", mode, tier),
                Some(&PromptOverride::Custom("legacy prompt".into()))
            );
        }
    }
}

#[test]
fn legacy_beaver_state_is_migrated_without_discarding_other_settings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    std::fs::write(
        &path,
        r#"{
          "global": {
            "chatbot": {
              "compact": { "state": "custom", "content": "global custom" },
              "detailed": null
            },
            "agentic": { "compact": null, "detailed": null }
          },
          "ollama": {
            "gemma4:e2b": {
              "chatbot": { "compact": { "state": "beaver" }, "detailed": null },
              "agentic": { "compact": null, "detailed": null }
            }
          }
        }"#,
    )
    .unwrap();

    let settings = SystemPromptSettings::read_from_path(&path).unwrap();

    assert_eq!(
        settings.global_override(PromptMode::Chatbot, PromptTier::Compact),
        Some(&PromptOverride::Custom("global custom".into()))
    );
    let restored = resolve_ollama(
        &settings,
        "gemma4:e2b",
        PromptMode::Chatbot,
        PromptTier::Compact,
        Some("native"),
        "beaver",
    );
    assert_eq!(restored.selection, PromptSelection::Beaver);
    assert_eq!(restored.content, "beaver");
}

#[test]
fn legacy_global_beaver_marker_is_discarded_because_global_already_defaults_to_beaver() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    std::fs::write(
        &path,
        r#"{
          "global": {
            "chatbot": { "compact": { "state": "beaver" }, "detailed": null },
            "agentic": { "compact": null, "detailed": null }
          },
          "ollama": {}
        }"#,
    )
    .unwrap();

    let settings = SystemPromptSettings::read_from_path(&path).unwrap();
    settings.write_to_path(&path).unwrap();

    let serialized = std::fs::read_to_string(path).unwrap();
    assert!(!serialized.contains(r#""compact_beaver": true"#));
}

#[test]
fn corrupt_system_prompt_settings_are_unavailable_and_never_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    let corrupt = b"{not valid json";
    std::fs::write(&path, corrupt).unwrap();
    let store = SystemPromptSettingsStore::open(path.clone(), legacy_path);

    assert_eq!(
        store.snapshot().err(),
        Some("system-prompt-store-unavailable".to_string())
    );
    assert_eq!(
        store.save_global(PromptMode::Chatbot, PromptTier::Compact, "replacement"),
        Err("system-prompt-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn runtime_prompt_composition_falls_back_to_defaults_when_settings_are_corrupt() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(&path, b"{not valid json").unwrap();
    let store = SystemPromptSettingsStore::open(path, legacy_path);

    let settings = store.snapshot_for_runtime();

    assert_eq!(
        settings.global_override(PromptMode::Chatbot, PromptTier::Compact),
        None
    );
}

#[test]
fn unavailable_system_prompt_store_recovers_after_valid_settings_are_restored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(&path, b"{not valid json").unwrap();
    let store = SystemPromptSettingsStore::open(path.clone(), legacy_path);
    let mut restored = SystemPromptSettings::default();
    restored
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "restored")
        .unwrap();
    restored.write_to_path(&path).unwrap();

    assert_eq!(
        store
            .snapshot()
            .unwrap()
            .global_override(PromptMode::Chatbot, PromptTier::Compact),
        Some(&PromptOverride::Custom("restored".to_string()))
    );
}

#[test]
fn system_prompt_store_does_not_overwrite_corruption_that_happens_after_open() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    let store = SystemPromptSettingsStore::open(path.clone(), legacy_path);
    store
        .save_global(PromptMode::Chatbot, PromptTier::Compact, "first")
        .unwrap();
    let corrupt = b"{corrupted while Beaver is running";
    std::fs::write(&path, corrupt).unwrap();

    assert_eq!(
        store.save_global(PromptMode::Chatbot, PromptTier::Detailed, "second"),
        Err("system-prompt-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn deleted_current_settings_do_not_resurrect_stale_legacy_prompts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(
        &legacy_path,
        br#"{"prompts":{"gemma4:e2b":"stale legacy"}}"#,
    )
    .unwrap();
    let store = SystemPromptSettingsStore::open(path.clone(), legacy_path);
    store
        .save_global(PromptMode::Chatbot, PromptTier::Compact, "current")
        .unwrap();
    std::fs::remove_file(&path).unwrap();

    assert_eq!(
        store.save_global(PromptMode::Chatbot, PromptTier::Detailed, "new"),
        Err("system-prompt-store-missing".to_string())
    );
    assert!(!path.exists());
}

#[test]
fn successful_legacy_migration_is_retired_before_a_later_restart() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(
        &legacy_path,
        br#"{"prompts":{"gemma4:e2b":"legacy prompt"}}"#,
    )
    .unwrap();

    let first = SystemPromptSettings::read_with_legacy(&path, &legacy_path).unwrap();
    assert_eq!(
        first.ollama_override(
            "gemma4:e2b",
            PromptMode::Chatbot,
            PromptTier::Compact,
        ),
        Some(&PromptOverride::Custom("legacy prompt".to_string()))
    );
    assert!(!legacy_path.exists());

    std::fs::remove_file(&path).unwrap();
    let restarted = SystemPromptSettingsStore::open(path, legacy_path);
    assert_eq!(
        restarted
            .snapshot()
            .unwrap()
            .ollama_override(
                "gemma4:e2b",
                PromptMode::Chatbot,
                PromptTier::Compact,
            ),
        None
    );
}

#[test]
fn stale_legacy_file_is_retired_when_current_settings_already_exist() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    let mut current = SystemPromptSettings::default();
    current
        .set_global(PromptMode::Chatbot, PromptTier::Compact, "current")
        .unwrap();
    current.write_to_path(&path).unwrap();
    std::fs::write(
        &legacy_path,
        br#"{"prompts":{"gemma4:e2b":"stale legacy"}}"#,
    )
    .unwrap();

    let loaded = SystemPromptSettings::read_with_legacy(&path, &legacy_path).unwrap();

    assert_eq!(
        loaded.global_override(PromptMode::Chatbot, PromptTier::Compact),
        Some(&PromptOverride::Custom("current".to_string()))
    );
    assert!(!legacy_path.exists());
}

#[test]
#[cfg(unix)]
fn failed_legacy_migration_write_reports_a_write_error() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let legacy_parent = directory.path().join("read-only-legacy");
    std::fs::create_dir(&legacy_parent).unwrap();
    let legacy_path = legacy_parent.join("ollama-system-prompts.json");
    std::fs::write(&legacy_path, br#"{"prompts":{}}"#).unwrap();
    std::fs::set_permissions(&legacy_parent, std::fs::Permissions::from_mode(0o500)).unwrap();
    let path = directory.path().join("system-prompt-settings.json");

    let result = SystemPromptSettings::read_with_legacy(&path, &legacy_path).err();
    std::fs::set_permissions(&legacy_parent, std::fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(result, Some("system-prompt-store-write".to_string()));
}

#[test]
fn corrupt_legacy_system_prompt_settings_are_not_silently_discarded() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system-prompt-settings.json");
    let legacy_path = directory.path().join("ollama-system-prompts.json");
    std::fs::write(&legacy_path, b"{not valid json").unwrap();

    assert_eq!(
        SystemPromptSettings::read_with_legacy(&path, &legacy_path).err(),
        Some("system-prompt-store-unavailable".to_string())
    );
    assert!(!path.exists());
}
