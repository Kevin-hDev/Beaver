use super::CodexTokens;
use chrono::Utc;
use zeroize::Zeroizing;

/// Construit un CodexTokens avec expires_at contrôlé.
fn tokens_with_expiry(expires_at: i64) -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access-token".to_string()),
        refresh: Zeroizing::new("refresh-token".to_string()),
        expires_at,
        account_hint: Zeroizing::new("acct_123".to_string()),
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
