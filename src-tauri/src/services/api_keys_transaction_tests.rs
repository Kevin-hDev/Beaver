use super::*;
use subtle::ConstantTimeEq;

fn state_with_old_secret() -> VaultState {
    VaultState {
        master_key: Zeroizing::new(vec![7_u8; 32]),
        keys: HashMap::from([(
            "openai".to_string(),
            Zeroizing::new("old-secret".to_string()),
        )]),
    }
}

#[test]
fn failed_persistence_keeps_previous_memory_state() {
    let mut state = state_with_old_secret();
    let result = commit_candidate_with(
        &mut state,
        |candidate| {
            candidate.insert("openai".to_string(), "new-secret".to_string());
            Ok(())
        },
        |_, _| Err("écriture refusée".to_string()),
    );

    assert!(result.is_err());
    assert!(bool::from(
        state
            .keys
            .get("openai")
            .unwrap()
            .as_bytes()
            .ct_eq(b"old-secret")
    ));
}

#[test]
fn successful_persistence_replaces_memory_after_write() {
    let mut state = state_with_old_secret();
    let result = commit_candidate_with(
        &mut state,
        |candidate| {
            candidate.insert("openai".to_string(), "new-secret".to_string());
            Ok(())
        },
        |_, candidate| {
            assert!(bool::from(
                candidate
                    .get("openai")
                    .unwrap()
                    .as_bytes()
                    .ct_eq(b"new-secret")
            ));
            Ok(())
        },
    );

    assert!(result.is_ok());
    assert!(bool::from(
        state
            .keys
            .get("openai")
            .unwrap()
            .as_bytes()
            .ct_eq(b"new-secret")
    ));
}

#[test]
fn raw_presence_lookup_uses_the_vault_namespace() {
    assert_eq!(prefixed_raw_key("oauth").unwrap(), "raw:oauth");
    assert!(prefixed_raw_key("").is_err());
    assert!(prefixed_raw_key(&"x".repeat(MAX_RAW_KEY_LEN + 1)).is_err());
}

#[test]
fn api_key_and_scope_are_committed_as_one_candidate() {
    let mut state = VaultState {
        master_key: Zeroizing::new(vec![7_u8; 32]),
        keys: HashMap::new(),
    };

    let first_scope = generate_credential_scope().unwrap();
    commit_candidate_with(
        &mut state,
        |candidate| stage_api_key(candidate, "openai", Some("first"), Some(&first_scope)),
        |_, _| Ok(()),
    )
    .unwrap();

    let scope_key = prefixed_raw_key("reasoning_scope:openai").unwrap();
    assert!(bool::from(
        state.keys.get("openai").unwrap().as_bytes().ct_eq(b"first")
    ));
    assert_eq!(
        state.keys.get(&scope_key).map(|value| value.as_str()),
        Some(first_scope.as_str())
    );

    let second_scope = generate_credential_scope().unwrap();
    commit_candidate_with(
        &mut state,
        |candidate| stage_api_key(candidate, "openai", Some("second"), Some(&second_scope)),
        |_, _| Ok(()),
    )
    .unwrap();
    assert!(bool::from(
        state
            .keys
            .get("openai")
            .unwrap()
            .as_bytes()
            .ct_eq(b"second")
    ));
    assert_eq!(
        state.keys.get(&scope_key).map(|value| value.as_str()),
        Some(second_scope.as_str())
    );
    assert_ne!(first_scope, second_scope);

    let rejected_scope = generate_credential_scope().unwrap();
    let failed = commit_candidate_with(
        &mut state,
        |candidate| stage_api_key(candidate, "openai", Some("rejected"), Some(&rejected_scope)),
        |_, _| Err("write refused".to_string()),
    );

    assert!(failed.is_err());
    assert!(bool::from(
        state
            .keys
            .get("openai")
            .unwrap()
            .as_bytes()
            .ct_eq(b"second")
    ));
    assert_eq!(
        state.keys.get(&scope_key).map(|value| value.as_str()),
        Some(second_scope.as_str())
    );

    commit_candidate_with(
        &mut state,
        |candidate| stage_api_key(candidate, "openai", None, None),
        |_, _| Ok(()),
    )
    .unwrap();
    assert!(!state.keys.contains_key("openai"));
    assert!(!state.keys.contains_key(&scope_key));
}

#[test]
fn candidate_validation_rejects_oversized_raw_values_before_persistence() {
    let mut state = state_with_old_secret();
    let persisted = std::cell::Cell::new(false);

    let result = commit_candidate_with(
        &mut state,
        |candidate| {
            candidate.insert(
                prefixed_raw_key("oversized")?,
                "v".repeat(MAX_RAW_VALUE_LEN + 1),
            );
            Ok(())
        },
        |_, _| {
            persisted.set(true);
            Ok(())
        },
    );

    assert!(result.is_err());
    assert!(!persisted.get());
    assert!(!state.keys.contains_key("raw:oversized"));
}

#[test]
fn key_and_provider_configuration_share_one_candidate_transaction() {
    let mut state = state_with_old_secret();
    let config_key = "provider_connection:qwen";

    let failed = commit_candidate_with(
        &mut state,
        |candidate| {
            stage_api_key(candidate, "qwen", Some("new-secret"), None)?;
            stage_raw_entries(candidate, &[(config_key, "new-config")])
        },
        |_, _| Err("write refused".to_string()),
    );
    assert!(failed.is_err());
    assert!(bool::from(
        state
            .keys
            .get("openai")
            .unwrap()
            .as_bytes()
            .ct_eq(b"old-secret")
    ));
    assert!(!state.keys.contains_key("qwen"));
    assert!(!state.keys.contains_key("raw:provider_connection:qwen"));
}
