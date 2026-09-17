use super::*;
use crate::services::agent_local::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextTokenCount,
};

#[tokio::test]
async fn rejected_http_response_returns_one_stable_public_code() {
    use crate::services::agent_local::types_ollama::OllamaThink;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(400).set_body_string("private upstream detail"))
        .mount(&server)
        .await;
    let response = reqwest::get(server.uri()).await.unwrap();
    let request = super::super::agent_loop_support::build_request(
        "fixture",
        &[],
        &[],
        OllamaThink::Bool(false),
    );
    let emitter = super::super::stream_events::AgentEventEmitter::test("session".into());

    let error = match handle_http_failure(
        &emitter,
        &request,
        response,
        &tokio_util::sync::CancellationToken::new(),
        RetryCounts {
            parser_retries: 0,
            server_retries: 0,
        },
        false,
    )
    .await
    {
        Err(error) => error,
        Ok(_) => panic!("HTTP 400 must fail"),
    };

    assert_eq!(error, request_error::SERVER);
    assert!(!error.contains("private upstream detail"));
}

#[tokio::test]
async fn cancellation_stays_distinct_from_transport_failures() {
    let cancel = tokio_util::sync::CancellationToken::new();
    cancel.cancel();

    assert_eq!(
        request_error::wait_retry(&cancel, 1).await.unwrap_err(),
        "Annulé"
    );
}

#[test]
fn malformed_response_and_timeout_have_stable_public_codes() {
    let code = request_error::invalid_response("JSON invalide: private upstream detail");

    assert_eq!(code, request_error::INVALID_RESPONSE);
    assert!(!code.contains("private upstream detail"));
    assert_eq!(request_error::TIMEOUT, "timeout");
}

#[tokio::test]
async fn unverified_local_count_is_not_persisted_as_in_flight() {
    let session = super::super::session_store::create_full(
        "Unknown local context",
        "fixture",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    let journal = super::super::conversation_journal::ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    journal.activate_context_request().await.unwrap();
    let emitter = super::super::stream_events::AgentEventEmitter::test(session.id.clone());
    let attempt = super::super::context_usage_runtime::PreparedContextAttempt::new(
        super::super::context_usage_runtime::ContextAttempt {
            on_event: &emitter,
            journal: Some(&journal),
            provider_id: "ollama",
            model: "fixture",
            turn: 0,
            attempt: 1,
            context_limit: 32_000,
            measured_input_source: ContextCountSource::NativeCounter,
        },
        Default::default(),
    );
    let unknown = ContextTokenCount {
        tokens: None,
        capacity_tokens: None,
        source: None,
        coverage: ContextCountCoverage::Unknown,
    };

    assert_eq!(
        persist_verified_context(&unknown, Some(&attempt))
            .await
            .unwrap_err(),
        super::super::context_capacity_error::UNVERIFIED_CODE
    );
    let saved = super::super::session_store::get(&session.id).await.unwrap();
    assert!(saved.context_usage.current_preparation.is_none());
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
