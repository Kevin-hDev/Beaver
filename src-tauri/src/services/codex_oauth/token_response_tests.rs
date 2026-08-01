use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use super::*;
use crate::services::codex_oauth::token::constant_time_secret_eq;

fn access_token(exp: i64) -> String {
    let header = URL_SAFE_NO_PAD.encode(b"{}");
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&serde_json::json!({
            "exp": exp,
            "https://api.openai.com/auth": {"chatgpt_account_id": "acct_test"}
        }))
        .unwrap(),
    );
    format!("{header}.{payload}.signature")
}

fn current() -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new(access_token(1_900_000_000)),
        refresh: Zeroizing::new("old-refresh".to_string()),
        expires_at: 1_900_000_000,
        account_hint: Zeroizing::new("acct_test".to_string()),
    }
}

#[test]
fn exchange_prefers_the_jwt_expiration() {
    let response = CodexTokenResponse {
        access_token: Some(access_token(1_900_000_000)),
        refresh_token: Some("refresh".to_string()),
        expires_in: Some(1),
    };

    let tokens = from_exchange(response).unwrap();

    assert_eq!(tokens.expires_at, 1_900_000_000);
}

#[test]
fn refresh_preserves_fields_omitted_by_the_server() {
    let response = CodexTokenResponse {
        access_token: None,
        refresh_token: None,
        expires_in: None,
    };

    let tokens = from_refresh(response, &current()).unwrap();

    assert!(constant_time_secret_eq(
        tokens.access.as_bytes(),
        current().access.as_bytes()
    ));
    assert!(constant_time_secret_eq(
        tokens.refresh.as_bytes(),
        b"old-refresh"
    ));
    assert_eq!(tokens.expires_at, 1_900_000_000);
}

#[test]
fn refresh_rejects_an_explicit_empty_token() {
    let response = CodexTokenResponse {
        access_token: Some(String::new()),
        refresh_token: None,
        expires_in: None,
    };

    assert!(from_refresh(response, &current()).is_err());
}
