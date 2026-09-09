use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

async fn mapped_error(status: u16, body: &str) -> crate::services::llm::types::LlmError {
    let response = tauri::http::Response::builder()
        .status(status)
        .body(body.to_string())
        .expect("valid fixture response");
    super::map_error_status(
        reqwest::Response::from(response),
        crate::services::llm::route_profile::ErrorPolicy::OpenAiCompatible,
    )
    .await
}

#[tokio::test]
async fn catalog_http_statuses_keep_api_categories() {
    use crate::services::llm::provider_error::ProviderErrorCode;
    use crate::services::llm::types::LlmError;

    assert!(matches!(
        mapped_error(401, "{}").await,
        LlmError::Unauthorized
    ));
    assert!(matches!(
        mapped_error(403, r#"{"error":{"type":"labs_not_enabled","code":1913}}"#).await,
        LlmError::KnownProvider(ProviderErrorCode::ProviderAccessUnavailable)
    ));
    assert!(matches!(
        mapped_error(429, "{}").await,
        LlmError::RateLimit {
            retry_after_secs: None
        }
    ));
}

#[tokio::test]
async fn authenticated_provider_client_refuses_redirects() {
    let destination = MockServer::start().await;
    let origin = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/sink", destination.uri())),
        )
        .mount(&origin)
        .await;
    let provider = OpenAiCompatProvider::new("openai").unwrap();

    let request = provider
        .client
        .post(format!("{}/chat", origin.uri()))
        .bearer_auth("fixture-secret")
        .body("fixture-body");
    let result = provider.client.send(request).await;

    assert!(result.is_err());
    assert!(destination.received_requests().await.unwrap().is_empty());
}
