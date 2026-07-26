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

// --- is_expired : marge de 30s ----------------------------------------------

#[test]
fn is_expired_true_when_past_expiry() {
    let now = Utc::now().timestamp();
    // Expiré il y a 1h.
    let t = tokens_with_expiry(now - 3600);
    assert!(t.is_expired());
}

#[test]
fn is_expired_false_when_well_before_expiry() {
    let now = Utc::now().timestamp();
    // Expire dans 1h.
    let t = tokens_with_expiry(now + 3600);
    assert!(!t.is_expired());
}

#[test]
fn is_expired_true_within_30s_refresh_margin() {
    // La marge de 30s évite d'utiliser un token qui va expirer pendant un
    // appel réseau en cours. expires_at - 30s doit déjà être considéré
    // expiré. On place expires_at 10s dans le futur : is_expired compare à
    // expires_at - 30 = -20s dans le passé → expiré.
    let now = Utc::now().timestamp();
    let t = tokens_with_expiry(now + 10);
    assert!(
        t.is_expired(),
        "un token qui expire dans <30s doit déjà être marqué expiré (refresh margin)"
    );
}

#[test]
fn is_expired_false_just_outside_refresh_margin() {
    // expires_at = now + 35s → expires_at - 30 = now + 5s → dans le futur →
    // pas expiré. Frontière exacte de la marge.
    let now = Utc::now().timestamp();
    let t = tokens_with_expiry(now + 35);
    assert!(!t.is_expired());
}

#[test]
fn is_expired_boundary_at_exact_expiry() {
    let now = Utc::now().timestamp();
    // expires_at = now → now >= expires_at - 30 → expiré.
    let t = tokens_with_expiry(now);
    assert!(t.is_expired());
}
