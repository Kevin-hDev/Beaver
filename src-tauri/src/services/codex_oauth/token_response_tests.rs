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
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_test".to_string()),
        credential_scope: Some(
            crate::services::api_keys::generate_credential_scope().expect("scope"),
        ),
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
    assert_eq!(tokens.refresh_not_before, 0);
    assert!(tokens.credential_scope.is_some());
}

#[test]
fn a_new_exchange_rotates_the_local_credential_scope() {
    let response = || CodexTokenResponse {
        access_token: Some(access_token(1_900_000_000)),
        refresh_token: Some("refresh".to_string()),
        expires_in: Some(3_600),
    };

    let first = from_exchange(response()).unwrap();
    let second = from_exchange(response()).unwrap();

    assert_ne!(first.credential_scope, second.credential_scope);
}

#[test]
fn refresh_preserves_fields_omitted_by_the_server() {
    let response = CodexTokenResponse {
        access_token: None,
        refresh_token: None,
        expires_in: None,
    };

    let current = current();
    let tokens = from_refresh(response, &current).unwrap();

    assert!(constant_time_secret_eq(
        tokens.access.as_bytes(),
        current.access.as_bytes()
    ));
    assert!(constant_time_secret_eq(
        tokens.refresh.as_bytes(),
        b"old-refresh"
    ));
    assert_eq!(tokens.expires_at, 1_900_000_000);
    assert!(tokens.refresh_not_before > chrono::Utc::now().timestamp());
    assert_eq!(
        tokens.credential_scope.as_ref().map(|scope| scope.as_str()),
        current
            .credential_scope
            .as_ref()
            .map(|scope| scope.as_str())
    );
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

#[test]
fn legacy_refresh_creates_a_scope_that_stays_stable() {
    let mut legacy = current();
    legacy.credential_scope = None;
    let first = from_refresh(
        CodexTokenResponse {
            access_token: None,
            refresh_token: None,
            expires_in: None,
        },
        &legacy,
    )
    .unwrap();
    assert!(first.credential_scope.is_some());

    let second = from_refresh(
        CodexTokenResponse {
            access_token: None,
            refresh_token: None,
            expires_in: None,
        },
        &first,
    )
    .unwrap();
    assert_eq!(second.credential_scope, first.credential_scope);
}
