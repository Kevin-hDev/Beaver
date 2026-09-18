use super::{validation::validate_model_with_ollama, AutomationError};
use crate::services::agent_local::ollama_client::OllamaClient;
use serde_json::json;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn ollama_routing_uses_local_capabilities_without_a_cloud_catalogue() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_json(
            json!({"model": "local-routing-regression:latest"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "capabilities": ["completion", "tools"]
        })))
        .mount(&server)
        .await;
    let result = validate_model_with_ollama("ollama", "local-routing-regression:latest", &|| {
        OllamaClient::with_base_url(&server.uri())
    })
    .await;
    assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn ollama_routing_does_not_accept_a_model_without_tools() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "capabilities": ["completion"]
        })))
        .mount(&server)
        .await;
    assert_eq!(
        validate_model_with_ollama("ollama", "text-only:latest", &|| {
            OllamaClient::with_base_url(&server.uri())
        })
        .await,
        Err(AutomationError::ModelToolsUnsupported)
    );
}

#[tokio::test]
async fn unavailable_ollama_client_is_not_reported_as_a_missing_model() {
    assert_eq!(
        validate_model_with_ollama("ollama", "installed:latest", &|| {
            Err("unavailable test application state".into())
        })
        .await,
        Err(AutomationError::ProviderUnavailable)
    );
}

#[tokio::test]
async fn other_routes_do_not_request_an_ollama_client() {
    assert_eq!(
        validate_model_with_ollama("codex-oauth", "gpt-5.6-luna", &|| {
            panic!("a non-Ollama route must not depend on the local engine")
        })
        .await,
        Ok(())
    );
    assert_eq!(
        validate_model_with_ollama("unknown-provider", "any", &|| {
            panic!("an invalid provider must be rejected before resolving a client")
        })
        .await,
        Err(AutomationError::ProviderUnavailable)
    );
}
