use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

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
