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
        (200, json!({}), AutomationError::ProviderUnavailable),
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

#[tokio::test]
async fn stopped_ollama_is_a_provider_failure_not_a_missing_model() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let client = OllamaClient::with_base_url(&format!("http://{address}")).unwrap();
    assert_eq!(
        validate_model(&client, "installed:latest").await,
        Err(AutomationError::ProviderUnavailable)
    );
}

#[tokio::test]
async fn failed_ollama_service_is_not_reported_as_an_absent_model() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    let client = OllamaClient::with_base_url(&server.uri()).unwrap();
    assert_eq!(
        validate_model(&client, "installed:latest").await,
        Err(AutomationError::ProviderUnavailable)
    );
}

#[tokio::test]
async fn disconnected_ollama_response_is_a_provider_failure() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 4096];
        let received = socket.read(&mut request).await.unwrap();
        assert!(received > 0);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n{")
            .await
            .unwrap();
        socket.shutdown().await.unwrap();
    });
    let client = OllamaClient::with_base_url(&endpoint).unwrap();
    assert_eq!(
        validate_model(&client, "installed:latest").await,
        Err(AutomationError::ProviderUnavailable)
    );
    server.await.unwrap();
}

#[tokio::test]
async fn stalled_ollama_validation_finishes_before_the_general_client_deadline() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"capabilities": ["tools"]}))
                .set_delay(std::time::Duration::from_secs(10)),
        )
        .mount(&server)
        .await;
    let client = OllamaClient::with_base_url(&server.uri()).unwrap();
    let outcome = tokio::time::timeout(
        std::time::Duration::from_secs(7),
        validate_model(&client, "installed:latest"),
    )
    .await
    .expect("automation validation must not wait for the 30-second client deadline");
    assert_eq!(outcome, Err(AutomationError::ProviderUnavailable));
}
