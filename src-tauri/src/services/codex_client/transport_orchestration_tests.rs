#[path = "transport_orchestration_tests/http.rs"]
mod http;
#[path = "transport_orchestration_tests/websocket.rs"]
mod websocket_cases;

use super::test_transport::{HttpCapture, WebSocketCapture};
use crate::services::agent_local::types_ollama::ChatMessage;

fn messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "user".to_string(),
        content: "bonjour".to_string(),
        ..Default::default()
    }]
}

fn assert_http_capture(
    capture: &HttpCapture,
    model: &str,
    service_tier: Option<&str>,
    routing_hint: &str,
) {
    assert_body_and_hint(
        &capture.request,
        capture.routing_hint.as_deref(),
        model,
        service_tier,
        routing_hint,
    );
    assert!(capture.authorization_valid);
    assert!(capture.account_header_present);
    assert!(capture.originator_valid);
    assert!(capture.user_agent_present);
    assert!(capture.response_path_valid);
    assert!(!capture.request.forbidden_field_present);
    assert!(capture.request.body_bytes <= crate::services::secure_http::LLM_BODY_LIMIT);
    assert!(capture.request.input_count <= 2_048);
    assert!(capture.request.tool_count <= 2_048);
}

fn assert_websocket_capture(
    capture: &WebSocketCapture,
    model: &str,
    service_tier: Option<&str>,
    routing_hint: &str,
) {
    assert_body_and_hint(
        &capture.request,
        capture.routing_hint.as_deref(),
        model,
        service_tier,
        routing_hint,
    );
    assert_eq!(
        capture.request.envelope_type.as_deref(),
        Some("response.create")
    );
    assert!(capture.authorization_valid);
    assert!(capture.account_header_present);
    assert!(capture.originator_valid);
    assert!(capture.user_agent_present);
    assert!(capture.beta_header_valid);
    assert!(capture.session_headers_valid);
    assert!(!capture.request.forbidden_field_present);
    assert!(capture.request.body_bytes <= crate::services::secure_http::LLM_BODY_LIMIT);
    assert!(capture.request.input_count <= 2_048);
    assert!(capture.request.tool_count <= 2_048);
}

fn assert_body_and_hint(
    request: &super::test_transport::RequestProjection,
    actual_hint: Option<&str>,
    model: &str,
    service_tier: Option<&str>,
    expected_hint: &str,
) {
    assert_eq!(request.model, model);
    assert_eq!(request.service_tier.as_deref(), service_tier);
    assert_eq!(actual_hint, Some(expected_hint));
    assert!(actual_hint.unwrap().len() <= 160);
}
