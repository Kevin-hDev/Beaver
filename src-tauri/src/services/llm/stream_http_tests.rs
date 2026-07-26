use std::time::Duration;

use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

#[tokio::test]
async fn chat_request_refuses_redirects_before_forwarding_the_body() {
    let destination = MockServer::start().await;
    let origin = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/sink", destination.uri())),
        )
        .mount(&origin)
        .await;

    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .expect("client");
    let result = client
        .send(
            client
                .post(format!("{}/chat", origin.uri()))
                .bearer_auth("fixture-secret")
                .json(&serde_json::json!({"private": "payload"})),
        )
        .await;

    assert!(result.is_err());
    assert!(destination
        .received_requests()
        .await
        .expect("requests")
        .is_empty());
}

#[tokio::test]
async fn oversized_provider_error_is_not_loaded() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(500).set_body_string(
                "x".repeat(crate::services::secure_http::PROVIDER_ERROR_LIMIT + 1),
            ),
        )
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .expect("client");
    let response = client
        .send(client.get(server.uri()))
        .await
        .expect("response");

    let body = read_provider_error(response).await;

    assert!(body.is_empty());
}

#[test]
fn gpt_56_uses_max_completion_tokens_in_chat_payload() {
    let cfg = RequestConfig {
        provider_id: "openai",
        model: "gpt-5.6-luna",
        messages: &[],
        tools: &[],
        think: true,
        reasoning_mode: Some("medium"),
        max_tokens: Some(32_000),
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
    };

    let route = route::resolve("openai").unwrap();
    let payload = build_chat_payload(&cfg, &route);

    assert_eq!(payload["max_completion_tokens"], 32_000);
    assert!(payload.get("max_tokens").is_none());
    assert_eq!(payload["reasoning_effort"], "medium");
}

#[test]
fn openrouter_gpt_56_uses_max_completion_tokens() {
    let cfg = RequestConfig {
        provider_id: "openrouter",
        model: "openai/gpt-5.6-sol",
        messages: &[],
        tools: &[],
        think: true,
        reasoning_mode: Some("medium"),
        max_tokens: Some(32_000),
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
    };

    let route = route::resolve("openrouter").unwrap();
    let payload = build_chat_payload(&cfg, &route);

    assert_eq!(payload["max_completion_tokens"], 32_000);
    assert!(payload.get("max_tokens").is_none());
}

#[test]
fn other_providers_keep_max_tokens() {
    let cfg = RequestConfig {
        provider_id: "xai",
        model: "grok-4.5",
        messages: &[],
        tools: &[],
        think: true,
        reasoning_mode: Some("medium"),
        max_tokens: Some(8_000),
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
    };

    let route = route::resolve("xai").unwrap();
    let payload = build_chat_payload(&cfg, &route);

    assert_eq!(payload["max_tokens"], 8_000);
    assert!(payload.get("max_completion_tokens").is_none());
}

#[test]
fn moonshot_membership_error_has_a_stable_safe_code() {
    let body = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active.","type":"invalid_request_error"}}"#;

    let error = classify_error(402, body, "Moonshot AI", "moonshot-oauth", true);

    assert_eq!(error.to_string(), "moonshot_membership_unverified");
}

#[test]
fn xai_spending_limit_error_has_a_stable_safe_code() {
    let body =
        r#"{"code":"personal-team-blocked:spending-limit","error":"private upstream details"}"#;

    let error = classify_error(402, body, "xAI", "xai-oauth", true);

    assert_eq!(error.to_string(), "xai_subscription_or_credits_required");
    assert!(!error.to_string().contains("private upstream details"));
}

#[test]
fn unknown_payment_error_stays_generic() {
    let error = classify_error(
        402,
        r#"{"error":{"message":"secret account detail"}}"#,
        "Provider",
        "unknown",
        true,
    );

    assert_eq!(error.to_string(), "provider_access_unavailable");
    assert!(!error.to_string().contains("secret account detail"));
}

#[test]
fn oauth_auth_and_rate_errors_use_frontend_codes() {
    assert_eq!(
        classify_error(401, "", "xAI", "xai-oauth", true).to_string(),
        "oauth_reauthentication_required"
    );
    assert_eq!(
        classify_error(403, "", "xAI", "xai-oauth", true).to_string(),
        "provider_access_unavailable"
    );
    assert_eq!(
        classify_error(429, "", "xAI", "xai-oauth", true).to_string(),
        "rate_limit"
    );
}

#[tokio::test]
async fn timeout_above_secure_limit_uses_a_stable_code() {
    let cfg = RequestConfig {
        provider_id: "openai",
        model: "gpt-5.6-luna",
        messages: &[],
        tools: &[],
        think: false,
        reasoning_mode: None,
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
    };
    let timeout = crate::services::secure_http::MAX_AUTHENTICATED_TIMEOUT + Duration::from_secs(1);

    let error = post_chat_request_with_timeout(&cfg, timeout)
        .await
        .expect_err("timeout must be rejected");

    assert_eq!(error.to_string(), "provider_configuration_invalid");
}
