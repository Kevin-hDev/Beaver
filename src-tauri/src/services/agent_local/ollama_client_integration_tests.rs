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

    let parameters = data.parameters.as_ref().expect("editable parameters");
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].value, "\"User:\"");
    assert_eq!(parameters[1].value, "Assistant: ");
    assert_eq!(data.parameter_error, None);
    assert!(data.modelfile.starts_with("FROM x\n"));
}

#[tokio::test]
async fn model_editor_keeps_raw_modelfile_when_parameter_summary_is_too_large() {
    let server = MockServer::start().await;
    let oversized = "x".repeat(1025);
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelfile": "FROM x\nPARAMETER stop oversized\n",
            "parameters": format!("stop                           {oversized}")
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let data = client
        .get_model_editor_data("oversized:latest")
        .await
        .expect("raw Modelfile remains available");

    assert_eq!(data.modelfile, "FROM x\nPARAMETER stop oversized\n");
    assert_eq!(data.parameters, None);
    assert_eq!(data.parameter_error.as_deref(), Some("ollama-invalid-response"));
}

#[tokio::test]
async fn model_editor_disables_editing_instead_of_truncating_stored_parameters() {
    let server = MockServer::start().await;
    let summary = (0..33)
        .map(|index| format!("stop                           \"stop-{index}\""))
        .collect::<Vec<_>>()
        .join("\n");
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelfile": "FROM x\n",
            "parameters": summary
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let data = client
        .get_model_editor_data("many-stops:latest")
        .await
        .expect("raw Modelfile remains available");

    assert_eq!(data.modelfile, "FROM x\n");
    assert_eq!(data.parameters, None);
    assert_eq!(data.parameter_error.as_deref(), Some("ollama-parameter-invalid"));
}

#[tokio::test]
async fn model_editor_rejects_decoded_control_characters_before_editing() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelfile": "FROM x\nPARAMETER stop safe\n",
            "parameters": "stop                           \"line\\rbreak\""
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let data = client
        .get_model_editor_data("control:latest")
        .await
        .expect("raw Modelfile remains available");

    assert_eq!(data.modelfile, "FROM x\nPARAMETER stop safe\n");
    assert_eq!(data.parameters, None);
    assert_eq!(data.parameter_error.as_deref(), Some("ollama-parameter-invalid"));
}

#[test]
fn ollama_test_client_rejects_non_loopback_urls() {
    assert!(OllamaClient::with_base_url("https://example.com").is_err());
}

#[tokio::test]
async fn client_uses_the_injected_runtime_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::with_base_url(&server.uri()).expect("loopback test server");
    let result = client.list_models().await;

    assert!(
        result.is_ok(),
        "managed client must follow the selected port"
    );
}
