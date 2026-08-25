use super::*;
use crate::services::reasoning_continuity::contract::{CredentialScope, RouteId};
use std::cell::Cell;
use subtle::ConstantTimeEq;

#[test]
fn ollama_scope_is_local_without_a_vault_entry() {
    let scope = credential_scope(RouteId::Ollama).expect("local scope");

    assert_eq!(scope.as_str(), CredentialScope::LOCAL_UNCREDENTIALED);
    assert_eq!(credential_scope_vault_key(RouteId::Ollama), None);
}

#[test]
fn authenticated_scope_uses_one_logical_raw_key_prefix() {
    assert_eq!(
        credential_scope_vault_key(RouteId::OpenAi).as_deref(),
        Some("reasoning_scope:openai")
    );
}

#[test]
fn generated_scopes_are_non_empty_and_rotate() {
    let first = generate_credential_scope().expect("first scope");
    let second = generate_credential_scope().expect("second scope");

    assert!(!first.as_str().is_empty());
    assert_ne!(first, second);
}

#[test]
fn groq_has_no_reasoning_scope_authority() {
    assert!(credential_scope(RouteId::Groq).is_err());
    assert_eq!(credential_scope_vault_key(RouteId::Groq), None);
}

#[test]
fn legacy_api_and_oauth_records_migrate_once_before_publication() {
    let xai_legacy = r#"{"access":"access-x","refresh":"refresh-x","expires_at":9}"#;
    let codex_legacy =
        r#"{"access":"access-c","refresh":"refresh-c","expires_at":9,"account_id":"acct_1"}"#;
    let mut map = HashMap::from([
        ("openai".to_string(), "api-secret".to_string()),
        (
            prefixed_raw_key(LLM_OAUTH_XAI_KEY).unwrap(),
            xai_legacy.to_string(),
        ),
        (
            prefixed_raw_key(CODEX_OAUTH_KEY).unwrap(),
            codex_legacy.to_string(),
        ),
    ]);
    let writes = Cell::new(0_u8);

    let first = commit_credential_scope_migration_with(&mut map, |candidate| {
        writes.set(writes.get() + 1);
        assert!(scope_from_map(candidate, RouteId::OpenAi).is_ok());
        assert!(scope_from_map(candidate, RouteId::XaiOauth).is_ok());
        assert!(scope_from_map(candidate, RouteId::CodexOauth).is_ok());
        Ok(())
    });

    assert!(first.blocked.is_empty());
    assert_eq!(writes.get(), 1);
    let api_scope = scope_from_map(&map, RouteId::OpenAi).unwrap();
    let xai_scope = scope_from_map(&map, RouteId::XaiOauth).unwrap();
    let codex_scope = scope_from_map(&map, RouteId::CodexOauth).unwrap();
    let xai_record = decode_llm_oauth_record(
        map.get(&prefixed_raw_key(LLM_OAUTH_XAI_KEY).unwrap())
            .unwrap(),
        RouteId::XaiOauth,
    )
    .unwrap();
    let codex_record = decode_codex_oauth_record(
        map.get(&prefixed_raw_key(CODEX_OAUTH_KEY).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(xai_record.schema_version, OAUTH_CREDENTIAL_SCHEMA_VERSION);
    assert_eq!(codex_record.schema_version, OAUTH_CREDENTIAL_SCHEMA_VERSION);

    let second = commit_credential_scope_migration_with(&mut map, |_| {
        writes.set(writes.get() + 1);
        Ok(())
    });

    assert!(second.changed.is_empty());
    assert_eq!(writes.get(), 1);
    assert_eq!(scope_from_map(&map, RouteId::OpenAi).unwrap(), api_scope);
    assert_eq!(scope_from_map(&map, RouteId::XaiOauth).unwrap(), xai_scope);
    assert_eq!(
        scope_from_map(&map, RouteId::CodexOauth).unwrap(),
        codex_scope
    );
}

#[test]
fn failed_oauth_migration_keeps_tokens_but_publishes_no_scope() {
    let legacy = Zeroizing::new(
        r#"{"access":"legacy-access","refresh":"legacy-refresh","expires_at":9}"#.to_string(),
    );
    let key = prefixed_raw_key(LLM_OAUTH_KIMI_KEY).unwrap();
    let mut map = HashMap::from([(key.clone(), legacy.to_string())]);

    let report =
        commit_credential_scope_migration_with(&mut map, |_| Err("write refused".to_string()));

    assert!(report.blocked.contains(&RouteId::MoonshotOauth));
    let persisted = map.get(&key).unwrap();
    assert!(bool::from(persisted.as_bytes().ct_eq(legacy.as_bytes())));
    let record = decode_llm_oauth_record(persisted, RouteId::MoonshotOauth).unwrap();
    assert!(bool::from(record.access.as_bytes().ct_eq(b"legacy-access")));
    assert!(record.credential_scope.is_none());
    assert!(scope_from_map(&map, RouteId::MoonshotOauth).is_err());
}

#[test]
fn oauth_logout_then_login_rotates_while_refresh_preserves_scope() {
    let first = generate_credential_scope().unwrap();
    let mut refreshed = new_llm_oauth_record(
        "new-access".to_string(),
        "same-refresh".to_string(),
        10,
        None,
        first.clone(),
    );
    refreshed.credential_scope = Some(first.clone());

    assert_eq!(refreshed.credential_scope.as_ref(), Some(&first));

    drop(refreshed);
    let after_logout_login = generate_credential_scope().unwrap();
    assert_ne!(after_logout_login, first);
}

#[test]
fn full_vault_blocks_only_the_missing_api_scope() {
    let mut map = HashMap::from([("openai".to_string(), "api-secret".to_string())]);
    for index in 0..(MAX_VAULT_ENTRIES - 1) {
        map.insert(format!("legacy-{index}"), "value".to_string());
    }
    let writes = Cell::new(0_u8);

    let report = commit_credential_scope_migration_with(&mut map, |_| {
        writes.set(writes.get() + 1);
        Ok(())
    });

    assert_eq!(map.len(), MAX_VAULT_ENTRIES);
    assert_eq!(writes.get(), 0);
    assert!(report.blocked.contains(&RouteId::OpenAi));
    assert!(scope_from_map(&map, RouteId::OpenAi).is_err());
}

#[test]
fn oversized_oauth_upgrade_stays_legacy_while_other_scope_persists() {
    let legacy = Zeroizing::new(format!(
        r#"{{"access":"{}","refresh":"{}","expires_at":9}}"#,
        "a".repeat(4_050),
        "r".repeat(4_050)
    ));
    assert!(legacy.len() <= MAX_RAW_VALUE_LEN);
    let oauth_key = prefixed_raw_key(LLM_OAUTH_KIMI_KEY).unwrap();
    let mut map = HashMap::from([
        ("openai".to_string(), "api-secret".to_string()),
        (oauth_key.clone(), legacy.to_string()),
    ]);
    let writes = Cell::new(0_u8);

    let report = commit_credential_scope_migration_with(&mut map, |candidate| {
        writes.set(writes.get() + 1);
        assert!(scope_from_map(candidate, RouteId::OpenAi).is_ok());
        Ok(())
    });

    assert_eq!(writes.get(), 1);
    assert!(report.blocked.contains(&RouteId::MoonshotOauth));
    assert!(scope_from_map(&map, RouteId::OpenAi).is_ok());
    assert!(bool::from(
        map.get(&oauth_key)
            .unwrap()
            .as_bytes()
            .ct_eq(legacy.as_bytes())
    ));
}

#[test]
fn raw_staging_validates_the_whole_batch_before_mutation() {
    let mut map = HashMap::new();
    let long_key = "k".repeat(MAX_RAW_KEY_LEN + 1);
    let invalid_key = [("valid", "value"), (long_key.as_str(), "value")];
    assert!(stage_raw_entries(&mut map, &invalid_key).is_err());
    assert!(map.is_empty());

    let long_value = "v".repeat(MAX_RAW_VALUE_LEN + 1);
    let invalid_value = [("valid", "value"), ("other", long_value.as_str())];
    assert!(stage_raw_entries(&mut map, &invalid_value).is_err());
    assert!(map.is_empty());
}
