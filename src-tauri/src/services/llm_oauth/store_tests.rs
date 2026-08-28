use super::*;
use crate::services::api_keys::{prefixed_raw_key, stage_raw_entries};
use std::collections::HashMap;
use subtle::ConstantTimeEq;

#[test]
fn rejects_oversized_tokens_before_storage() {
    let tokens = TokenBundle {
        access: Zeroizing::new("a".repeat(MAX_TOKEN_LEN + 1)),
        refresh: Zeroizing::new("r".to_string()),
        expires_at: 1,
        user_id: None,
        credential_scope: None,
    };
    assert!(validate(&tokens).is_err());
}

#[test]
fn providers_use_distinct_vault_entries() {
    assert_ne!(
        LlmOAuthProvider::Xai.vault_key(),
        LlmOAuthProvider::Kimi.vault_key()
    );
    assert!(LlmOAuthProvider::Xai.vault_key().starts_with('_'));
    assert!(LlmOAuthProvider::Kimi.vault_key().starts_with('_'));
}

#[test]
fn stale_generation_is_rejected_before_storage() {
    let provider = LlmOAuthProvider::Kimi;
    let tokens = TokenBundle {
        access: Zeroizing::new("access".to_string()),
        refresh: Zeroizing::new("refresh".to_string()),
        expires_at: 1,
        user_id: None,
        credential_scope: None,
    };
    let stale = generation(provider).saturating_add(1);
    assert!(save_if_generation(provider, &tokens, stale).is_err());
}

#[test]
fn login_rotates_scope_and_refresh_preserves_it() {
    let mut login = TokenBundle {
        access: Zeroizing::new("access".to_string()),
        refresh: Zeroizing::new("refresh".to_string()),
        expires_at: 1,
        user_id: None,
        credential_scope: None,
    };
    login.assign_new_credential_scope().unwrap();
    let first = login.credential_scope.clone().unwrap();

    let mut refreshed = TokenBundle {
        access: Zeroizing::new("new-access".to_string()),
        refresh: Zeroizing::new("refresh".to_string()),
        expires_at: 2,
        user_id: None,
        credential_scope: None,
    };
    refreshed.preserve_credential_scope_from(&login).unwrap();
    assert_eq!(refreshed.credential_scope.as_ref(), Some(&first));

    let mut next_login = refreshed;
    next_login.assign_new_credential_scope().unwrap();
    assert_ne!(next_login.credential_scope.as_ref(), Some(&first));
}

#[test]
fn legacy_xai_enrich_repairs_the_record_atomically() {
    let (legacy, mut current) = legacy_tokens(LlmOAuthProvider::Xai);
    current.user_id = Some(Zeroizing::new("xai-user".to_string()));
    current.ensure_credential_scope_for_persistence().unwrap();

    verify_legacy_repair(LlmOAuthProvider::Xai, &legacy, current);
}

#[test]
fn legacy_xai_refresh_repairs_the_record_atomically() {
    let (legacy, current) = legacy_tokens(LlmOAuthProvider::Xai);
    let refreshed = refreshed_from_legacy(&current);

    verify_legacy_repair(LlmOAuthProvider::Xai, &legacy, refreshed);
}

#[test]
fn legacy_kimi_refresh_repairs_the_record_atomically() {
    let (legacy, current) = legacy_tokens(LlmOAuthProvider::Kimi);
    let refreshed = refreshed_from_legacy(&current);

    verify_legacy_repair(LlmOAuthProvider::Kimi, &legacy, refreshed);
}

fn legacy_tokens(provider: LlmOAuthProvider) -> (Zeroizing<String>, TokenBundle) {
    let legacy = Zeroizing::new(
        r#"{"access":"legacy-access","refresh":"legacy-refresh","expires_at":9}"#.to_string(),
    );
    let current = load_record(provider, &legacy).unwrap();
    (legacy, current)
}

fn verify_legacy_repair(
    provider: LlmOAuthProvider,
    legacy: &Zeroizing<String>,
    mut repaired_tokens: TokenBundle,
) {
    let physical_key = prefixed_raw_key(provider.vault_key()).unwrap();
    let mut map = HashMap::from([(physical_key.clone(), legacy.to_string())]);

    let failed = save_record_with(provider, &repaired_tokens, |key, json| {
        let mut candidate = map.clone();
        stage_raw_entries(&mut candidate, &[(key, json)])?;
        Err("write refused".to_string())
    });
    assert!(failed.is_err());
    assert!(bool::from(
        map.get(&physical_key)
            .unwrap()
            .as_bytes()
            .ct_eq(legacy.as_bytes())
    ));

    save_record_with(provider, &repaired_tokens, |key, json| {
        stage_raw_entries(&mut map, &[(key, json)])
    })
    .unwrap();
    let repaired = load_record(provider, map.get(&physical_key).unwrap()).unwrap();
    assert!(repaired.credential_scope.is_some());
    assert_eq!(repaired.credential_scope, repaired_tokens.credential_scope);

    let stable_scope = repaired.credential_scope.clone();
    repaired_tokens
        .preserve_credential_scope_from(&repaired)
        .unwrap();
    assert_eq!(repaired_tokens.credential_scope, stable_scope);
    save_record_with(provider, &repaired_tokens, |key, json| {
        stage_raw_entries(&mut map, &[(key, json)])
    })
    .unwrap();
    let stable = load_record(provider, map.get(&physical_key).unwrap()).unwrap();
    assert_eq!(stable.credential_scope, stable_scope);
}

fn refreshed_from_legacy(current: &TokenBundle) -> TokenBundle {
    let mut refreshed = TokenBundle {
        access: Zeroizing::new("new-access".to_string()),
        refresh: Zeroizing::new("legacy-refresh".to_string()),
        expires_at: 10,
        user_id: None,
        credential_scope: None,
    };
    refreshed.preserve_credential_scope_from(current).unwrap();
    refreshed
}
