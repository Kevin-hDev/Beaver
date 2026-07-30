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
    let payload = build_chat_payload(&cfg, &route, Some(32_000));

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
    let payload = build_chat_payload(&cfg, &route, Some(32_000));

    assert_eq!(payload["max_completion_tokens"], 32_000);
    assert!(payload.get("max_tokens").is_none());
}

#[test]
fn streaming_output_limit_field_matches_model_family() {
    for (provider, model, expected, absent) in [
        ("openai", "o3", "max_completion_tokens", "max_tokens"),
        ("openai", "gpt-4o", "max_tokens", "max_completion_tokens"),
        ("moonshot", "kimi-k3", "max_completion_tokens", "max_tokens"),
        (
            "moonshot",
            "kimi-k2.7-code",
            "max_tokens",
            "max_completion_tokens",
        ),
        ("xai", "grok-4.5", "max_tokens", "max_completion_tokens"),
    ] {
        let cfg = RequestConfig {
            provider_id: provider,
            model,
            messages: &[],
            tools: &[],
            think: false,
            reasoning_mode: None,
            max_tokens: Some(8_000),
            purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        };
        let route = route::resolve(provider).unwrap();
        let payload = build_chat_payload(&cfg, &route, Some(8_000));

        assert_eq!(payload[expected], 8_000, "{provider}/{model}");
        assert!(payload.get(absent).is_none(), "{provider}/{model}");
    }
}

#[tokio::test]
async fn groq_and_cerebras_payloads_omit_automatic_limits() {
    for (provider, model) in [
        ("groq", "llama-3.3-70b-versatile"),
        ("cerebras", "gpt-oss-120b"),
    ] {
        let route = route::resolve(provider).unwrap();
        let resolved = super::super::stream_max_tokens::resolve(
            provider,
            model,
            None,
            route.auto_max_tokens,
            route.fallback_max_tokens,
            0,
        )
        .await
        .unwrap();
        let cfg = RequestConfig {
            provider_id: provider,
            model,
            messages: &[],
            tools: &[],
            think: false,
            reasoning_mode: None,
            max_tokens: None,
            purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        };

        let payload = build_chat_payload(&cfg, &route, resolved);

        assert!(payload.get("max_tokens").is_none());
        assert!(payload.get("max_completion_tokens").is_none());
    }
}

#[tokio::test]
async fn openrouter_uses_the_underlying_model_output_limit() {
    for (model, expected) in [
        ("google/gemini-2.5-pro", 65_536),
        ("openai/gpt-4o", 16_384),
        ("openai/o3-mini", 100_000),
    ] {
        let route = route::resolve("openrouter").unwrap();
        let resolved = super::super::stream_max_tokens::resolve(
            "openrouter",
            model,
            None,
            route.auto_max_tokens,
            route.fallback_max_tokens,
            0,
        )
        .await
        .unwrap();

        assert_eq!(resolved, Some(expected));
    }
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
