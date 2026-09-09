use super::llm_probe;
use crate::services::llm::api_key_probe::{ProbeAuth, ProbeMethod};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn anthropic_uses_the_native_llm_probe_path() {
    let probe = llm_probe("anthropic")
        .expect("configurable LLM route")
        .expect("valid probe");
    assert_eq!(probe.method, ProbeMethod::Get);
    assert_eq!(probe.auth, ProbeAuth::XApiKey);
    assert_eq!(probe.headers, &[("anthropic-version", "2023-06-01")]);
}

#[test]
fn qwen_subscription_keys_are_rejected_before_network_use() {
    assert!(super::reject_unsupported_qwen_key("sk-sp-fixture").is_err());
}

#[test]
fn qwen_only_falls_back_to_chat_when_models_is_unsupported() {
    use super::QwenProbeAction;

    assert_eq!(super::qwen_probe_action(200), QwenProbeAction::Accept);
    assert_eq!(super::qwen_probe_action(404), QwenProbeAction::ChatFallback);
    assert_eq!(super::qwen_probe_action(405), QwenProbeAction::ChatFallback);
    assert_eq!(super::qwen_probe_action(401), QwenProbeAction::Reject);
    assert_eq!(super::qwen_probe_action(429), QwenProbeAction::Reject);
}

#[test]
fn a_stored_qwen_key_uses_the_connection_aware_probe() {
    assert!(super::uses_qwen_probe("qwen"));
    assert!(!super::uses_qwen_probe("anthropic"));
}

#[tokio::test]
async fn openrouter_key_probe_accepts_200_without_reading_account_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/key"))
        .and(header("authorization", "Bearer fixture-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":{"label":"must-not-be-logged","limit":99}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let mut probe = llm_probe("openrouter").unwrap().unwrap();
    probe.url = format!("{}/api/v1/key", server.uri());
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(
        std::time::Duration::from_secs(1),
    )
    .unwrap();

    let response = client
        .send(crate::services::llm::api_key_probe::request(
            &client,
            &probe,
            "fixture-secret",
        ))
        .await
        .unwrap();

    assert!(super::check_status(response).await.is_ok());
}

#[tokio::test]
async fn openrouter_key_probe_rejects_401_without_catalog_fallback() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/key"))
        .and(header("authorization", "Bearer invalid-fixture"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(
            r#"{"error":{"message":"sensitive account detail"}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let mut probe = llm_probe("openrouter").unwrap().unwrap();
    probe.url = format!("{}/api/v1/key", server.uri());
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(
        std::time::Duration::from_secs(1),
    )
    .unwrap();

    let response = client
        .send(crate::services::llm::api_key_probe::request(
            &client,
            &probe,
            "invalid-fixture",
        ))
        .await
        .unwrap();

    assert_eq!(
        super::check_status(response).await,
        Err("Clé API invalide ou non autorisée".to_string())
    );
}
