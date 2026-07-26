use super::*;
use crate::services::gateway::tokens::AccountTokens;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn slack_tokens() -> AccountTokens {
    AccountTokens {
        token: None,
        bot_token: Some("xoxb-test".into()),
        app_token: Some("xapp-test".into()),
    }
}

fn single_token(value: &str) -> AccountTokens {
    AccountTokens {
        token: Some(value.into()),
        bot_token: None,
        app_token: None,
    }
}

#[tokio::test]
async fn slack_validates_both_tokens() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth.test"))
        .and(header("authorization", "Bearer xoxb-test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"ok": true, "user_id": "U1"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/apps.connections.open"))
        .and(header("authorization", "Bearer xapp-test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"ok": true, "url": "wss://example.invalid"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let endpoints = ProbeEndpoints::all(server.uri());
    assert!(validate_tokens("slack", &slack_tokens(), &endpoints)
        .await
        .is_ok());
}

#[tokio::test]
async fn slack_rejects_when_the_app_token_fails() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth.test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"ok": true, "user_id": "U1"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/apps.connections.open"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"ok": false, "error": "invalid_auth"})),
        )
        .mount(&server)
        .await;

    let endpoints = ProbeEndpoints::all(server.uri());
    assert!(validate_tokens("slack", &slack_tokens(), &endpoints)
        .await
        .is_err());
}

#[tokio::test]
async fn telegram_and_discord_validate_real_identities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/bottg-fixture/getMe"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"ok": true, "result": {"id": "7"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/users/@me"))
        .and(header("authorization", "Bot dc-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "8"})))
        .expect(1)
        .mount(&server)
        .await;

    let endpoints = ProbeEndpoints::all(server.uri());
    assert!(
        validate_tokens("telegram", &single_token("tg-fixture"), &endpoints)
            .await
            .is_ok()
    );
    assert!(
        validate_tokens("discord", &single_token("dc-fixture"), &endpoints)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn probe_error_never_contains_the_telegram_credential() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let endpoints = ProbeEndpoints::all(server.uri());
    let fixture = "tg-private-fixture";
    let error = validate_tokens("telegram", &single_token(fixture), &endpoints)
        .await
        .unwrap_err();
    assert!(!error.contains(fixture));
}
