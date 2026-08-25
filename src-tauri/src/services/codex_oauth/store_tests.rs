use super::CodexTokens;
use chrono::Utc;
use zeroize::Zeroizing;

/// Construit un CodexTokens avec expires_at contrôlé.
fn tokens_with_expiry(expires_at: i64) -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access-token".to_string()),
        refresh: Zeroizing::new("refresh-token".to_string()),
        expires_at,
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_123".to_string()),
        credential_scope: Some(
            crate::services::api_keys::generate_credential_scope().expect("scope"),
        ),
    }
}

// --- renouvellement anticipé et expiration réelle --------------------------

#[test]
fn is_expired_true_when_past_expiry() {
    let now = Utc::now().timestamp();
    // Expiré il y a 1h.
    let t = tokens_with_expiry(now - 3600);
    assert!(t.is_expired());
    assert!(t.needs_refresh());
}

#[test]
fn is_expired_false_when_well_before_expiry() {
    let now = Utc::now().timestamp();
    // Expire dans 1h.
    let t = tokens_with_expiry(now + 3600);
    assert!(!t.is_expired());
    assert!(!t.needs_refresh());
}

#[test]
fn is_expired_true_within_refresh_margin() {
    let now = Utc::now().timestamp();
    let t = tokens_with_expiry(now + 120);
    assert!(
        t.needs_refresh(),
        "un token qui expire dans moins de cinq minutes doit être renouvelé"
    );
    assert!(!t.is_expired());
}

#[test]
fn is_expired_false_just_outside_refresh_margin() {
    let now = Utc::now().timestamp();
    let t = tokens_with_expiry(now + 305);
    assert!(!t.needs_refresh());
    assert!(!t.is_expired());
}

#[test]
fn is_expired_boundary_at_exact_expiry() {
    let now = Utc::now().timestamp();
    let t = tokens_with_expiry(now);
    assert!(t.is_expired());
}

#[test]
fn refresh_cooldown_prevents_a_network_refresh_storm() {
    let now = Utc::now().timestamp();
    let mut tokens = tokens_with_expiry(now + 120);
    tokens.refresh_not_before = now + 60;

    assert!(!tokens.needs_refresh());
}

#[test]
fn an_expired_token_ignores_the_refresh_cooldown() {
    let now = Utc::now().timestamp();
    let mut tokens = tokens_with_expiry(now - 1);
    tokens.refresh_not_before = now + 60;

    assert!(tokens.needs_refresh());
}

#[test]
fn legacy_storage_defaults_the_refresh_cooldown_to_zero() {
    let stored = crate::services::api_keys::decode_codex_oauth_record(
        r#"{"access":"a","refresh":"r","expires_at":1,"account_id":"acct_1"}"#,
    )
    .unwrap();

    assert_eq!(stored.refresh_not_before, 0);
}

#[test]
fn storage_failures_use_a_stable_public_error_code() {
    assert_eq!(super::unavailable(), "oauth_reauthentication_required");
}
