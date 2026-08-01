use super::{can_use_still_valid_token, constant_time_secret_eq, refresh_body};
use crate::services::codex_oauth::store::CodexTokens;
use zeroize::Zeroizing;

#[test]
fn secret_comparison_checks_content_and_length() {
    assert!(constant_time_secret_eq(b"token-value", b"token-value"));
    assert!(!constant_time_secret_eq(b"token-value", b"token-other"));
    assert!(!constant_time_secret_eq(
        b"token-value",
        b"token-value-long"
    ));
}

#[test]
fn refresh_request_uses_the_current_json_contract() {
    let body = refresh_body("refresh-value").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(parsed["grant_type"], "refresh_token");
    assert_eq!(parsed["client_id"], super::CLIENT_ID);
    assert!(constant_time_secret_eq(
        parsed["refresh_token"].as_str().unwrap().as_bytes(),
        b"refresh-value"
    ));
}

#[test]
fn proactive_refresh_falls_back_only_for_temporary_transport_failures() {
    let tokens = tokens_expiring_at(chrono::Utc::now().timestamp() + 60);

    assert!(can_use_still_valid_token(
        "provider_connection_failed",
        &tokens
    ));
    assert!(can_use_still_valid_token(
        "provider_temporarily_unavailable",
        &tokens
    ));
    assert!(!can_use_still_valid_token(
        "oauth_reauthentication_required",
        &tokens
    ));
}

#[test]
fn expired_token_never_survives_a_failed_refresh() {
    let tokens = tokens_expiring_at(chrono::Utc::now().timestamp() - 1);

    assert!(!can_use_still_valid_token(
        "provider_connection_failed",
        &tokens
    ));
}

fn tokens_expiring_at(expires_at: i64) -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access".to_string()),
        refresh: Zeroizing::new("refresh".to_string()),
        expires_at,
        account_hint: Zeroizing::new("account".to_string()),
    }
}
