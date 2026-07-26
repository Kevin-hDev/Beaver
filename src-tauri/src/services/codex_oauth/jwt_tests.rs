use super::*;

fn make_jwt(payload: &serde_json::Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(b"{}");
    let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap());
    let signature = URL_SAFE_NO_PAD.encode(b"sig");
    format!("{header}.{body}.{signature}")
}

#[test]
fn extracts_only_bounded_display_claims() {
    let jwt = make_jwt(&serde_json::json!({
        "https://api.openai.com/auth": {"chatgpt_account_id": "acct_abc123"},
        "https://api.openai.com/profile": {"email": "test@example.com"},
    }));
    let claims = extract_display_claims(&jwt).unwrap();
    assert_eq!(claims.account_hint, "acct_abc123");
    assert_eq!(claims.email.as_deref(), Some("test@example.com"));
}

#[test]
fn rejects_missing_or_header_unsafe_account_hint() {
    let missing = make_jwt(&serde_json::json!({"sub": "user123"}));
    assert!(extract_display_claims(&missing).is_err());
    let unsafe_id = make_jwt(&serde_json::json!({
        "https://api.openai.com/auth": {"chatgpt_account_id": "bad\r\nheader"}
    }));
    assert!(extract_display_claims(&unsafe_id).is_err());
}

#[test]
fn rejects_invalid_or_oversized_token() {
    assert!(extract_display_claims("not-a-jwt").is_err());
    assert!(extract_display_claims(&"a".repeat(MAX_JWT_BYTES + 1)).is_err());
}
