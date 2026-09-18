use super::{ollama_validation::validate_model, AutomationError};
use crate::services::agent_local::ollama_client::OllamaClient;
use serde_json::json;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn installed_ollama_tool_model_is_accepted_without_cloud_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_json(json!({"model": "qwen3.5:latest"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "capabilities": ["completion", "tools"]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = OllamaClient::with_base_url(&server.uri()).unwrap();
    assert_eq!(validate_model(&client, "qwen3.5:latest").await, Ok(()));
}

#[tokio::test]
async fn ollama_validation_fails_closed_on_absent_invalid_or_unsupported_models() {
    for (status, body, expected) in [
        (
            404,
            json!({"capabilities": ["tools"]}),
            AutomationError::ModelUnavailable,
        ),
        (200, json!({}), AutomationError::ModelUnavailable),
        (
            200,
            json!({"capabilities": ["completion"]}),
            AutomationError::ModelToolsUnsupported,
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/show"))
            .and(body_json(json!({"model": "test:latest"})))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let client = OllamaClient::with_base_url(&server.uri()).unwrap();
        assert_eq!(validate_model(&client, "test:latest").await, Err(expected));
    }
}

#[tokio::test]
async fn invalid_model_identifier_never_contacts_ollama() {
    let server = MockServer::start().await;
    let client = OllamaClient::with_base_url(&server.uri()).unwrap();
    assert_eq!(
        validate_model(&client, "bad\nmodel").await,
        Err(AutomationError::ModelUnavailable)
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}
