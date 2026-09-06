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
        plugin_descriptors: Vec::new(),
        active_plugin_ids: Vec::new(),
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
        plugin_descriptors: Vec::new(),
        active_plugin_ids: Vec::new(),
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

#[test]
fn sanitize_bounds_the_shared_plugin_descriptors() {
    let mut state = ExtensionSessionState {
        plugin_descriptors: vec![
            PluginDescriptor {
                id: "example.valid".to_string(),
                tool_count: 2,
                definition_count: 2,
            },
            PluginDescriptor {
                id: "bad id".to_string(),
                tool_count: 1,
                definition_count: 1,
            },
            PluginDescriptor {
                id: "example.valid".to_string(),
                tool_count: 3,
                definition_count: 3,
            },
        ],
        ..ExtensionSessionState::default()
    };

    sanitize(&mut state);

    assert_eq!(
        state.plugin_descriptors,
        vec![PluginDescriptor {
            id: "example.valid".to_string(),
            tool_count: 2,
            definition_count: 2,
        }]
    );
}

#[test]
fn legacy_discovery_state_with_missing_fields_remains_readable() {
    let state = parse_state(br#"{"discovered_plugin_ids":["example.one"]}"#);

    assert_eq!(state.discovered_plugin_ids, ["example.one"]);
    assert!(state.epoch.is_none());
    assert_eq!(state.plugin_tool_capacity, 0);
    assert!(state.plugin_descriptors.is_empty());
}

#[test]
fn legacy_discovery_state_ignores_extra_fields_and_sanitizes_identifiers() {
    let mut ids = vec![
        "example.first".to_string(),
        "bad id".to_string(),
        "example.first".to_string(),
        "example.second".to_string(),
    ];
    ids.extend(
        (0..crate::services::extensions::MAX_DISCOVERED_PLUGINS)
            .map(|index| format!("example.extra{index}")),
    );
    let historical = serde_json::json!({
        "discovered_plugin_ids": ids,
        "related_search_ids": ["legacy-correlation"],
        "legacy_discovery_cursor": {"offset": 12}
    });

    let state = parse_state(&serde_json::to_vec(&historical).unwrap());

    assert_eq!(
        state.discovered_plugin_ids.len(),
        crate::services::extensions::MAX_DISCOVERED_PLUGINS
    );
    assert_eq!(
        &state.discovered_plugin_ids[..2],
        ["example.first", "example.second"]
    );
    assert!(!state.discovered_plugin_ids.iter().any(|id| id == "bad id"));
}

#[tokio::test]
async fn failed_state_write_returns_the_public_code_without_overwriting_the_target() {
    let id = uuid::Uuid::new_v4().to_string();
    let target = path(&id);
    std::fs::create_dir_all(&target).unwrap();
    let error = mutate(&id, |_| Ok(())).await.unwrap_err();
    assert!(target.is_dir());
    std::fs::remove_dir(&target).unwrap();
    assert_eq!(
        error,
        crate::services::extensions::error_codes::STATE_UNAVAILABLE
    );
}
