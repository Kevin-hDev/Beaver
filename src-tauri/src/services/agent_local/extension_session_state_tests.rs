use super::*;

#[test]
fn epoch_key_ignores_the_frozen_mask_value() {
    let left = DiscoveryEpoch {
        provider: "openai".to_string(),
        model: "gpt".to_string(),
        context_window: 128_000,
        catalog_version: "a".repeat(64),
        masked: false,
    };
    let mut right = left.clone();
    right.masked = true;

    assert!(same_key(&left, &right));

    right.model = "other".to_string();
    assert!(!same_key(&left, &right));
}

#[test]
fn sanitize_bounds_and_deduplicates_discoveries() {
    let mut state = ExtensionSessionState {
        discovered_plugin_ids: vec![
            "example.one".to_string(),
            "example.one".to_string(),
            "bad id".to_string(),
        ],
        epoch: None,
        plugin_tool_capacity: usize::MAX,
    };

    sanitize(&mut state);

    assert_eq!(state.discovered_plugin_ids, vec!["example.one"]);
    assert_eq!(
        state.plugin_tool_capacity,
        crate::services::extensions::MAX_EXTENSION_TOOLS
    );
}

#[test]
fn sanitize_never_tracks_more_than_the_extension_limit() {
    let mut state = ExtensionSessionState {
        discovered_plugin_ids: (0..crate::services::extensions::MAX_DISCOVERED_PLUGINS + 2)
            .map(|index| format!("example.plugin{index}"))
            .collect(),
        epoch: None,
        plugin_tool_capacity: 0,
    };

    sanitize(&mut state);

    assert_eq!(
        state.discovered_plugin_ids.len(),
        crate::services::extensions::MAX_DISCOVERED_PLUGINS
    );
}

#[test]
fn invalid_epoch_metadata_is_rejected() {
    let epoch = DiscoveryEpoch {
        provider: "openai".to_string(),
        model: "gpt".to_string(),
        context_window: 128_000,
        catalog_version: "not-a-fingerprint".to_string(),
        masked: false,
    };

    assert!(invalid_epoch(&epoch));
}
