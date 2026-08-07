use super::ollama_client::OllamaClient;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn ollama_catalog_uses_the_real_http_contract() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [{
                "name": "tiny:latest",
                "size": 42,
                "digest": "sha256:abcdef1234567890",
                "details": { "family": "tiny", "parameter_size": "1B" }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_json(serde_json::json!({ "model": "tiny:latest" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelfile": "FROM tiny",
            "details": {
                "family": "tiny",
                "parameter_size": "1B",
                "quantization_level": "Q4"
            },
            "model_info": {
                "general.architecture": "tiny",
                "tiny.context_length": 8192
            },
            "capabilities": ["completion"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let models = client.list_models().await.expect("Ollama catalog");

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "tiny:latest");
    assert_eq!(models[0].context_length, 8192);
    assert_eq!(models[0].quantization, "Q4");
}

#[tokio::test]
async fn model_editor_data_uses_the_authoritative_parameter_summary() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_json(serde_json::json!({ "model": "quoted:latest" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelfile": concat!(
                "FROM x\n",
                "PARAMETER stop \"User:\"\n",
                "PARAMETER stop \"Assistant: \"\n"
            ),
            "parameters": concat!(
                "stop                           \"\\\"User:\\\"\"\n",
                "stop                           \"Assistant: \""
            )
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let data = client
        .get_model_editor_data("quoted:latest")
        .await
        .expect("model editor data");

    assert_eq!(data.parameters.len(), 2);
    assert_eq!(data.parameters[0].value, "\"User:\"");
    assert_eq!(data.parameters[1].value, "Assistant: ");
    assert!(data.modelfile.starts_with("FROM x\n"));
}

#[test]
fn ollama_test_client_rejects_non_loopback_urls() {
    assert!(OllamaClient::with_base_url("https://example.com").is_err());
}

#[tokio::test]
async fn client_created_before_port_selection_follows_the_runtime_port() {
    let _guard = crate::services::ollama_port::PORT_TEST_LOCK.lock().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    crate::services::ollama_port::set_port(0);
    let client = OllamaClient::new();
    crate::services::ollama_port::set_port(server.address().port());
    let result = client.list_models().await;
    crate::services::ollama_port::set_port(0);

    assert!(
        result.is_ok(),
        "managed client must follow the selected port"
    );
}
