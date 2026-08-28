use super::xai_oauth_transport::{
    backend_path, catalog_reasoning_mode, classify_status, prepare_chat_request, validate_backend,
};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_http::RequestConfig;
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};

fn catalog_model() -> XaiCatalogModel {
    XaiCatalogModel {
        id: "grok-4.6".to_string(),
        display_name: "Grok 4.6".to_string(),
        backend: XaiBackend::Responses,
        context_window: 500_000,
        max_output_tokens: None,
        reasoning_modes: vec!["low".into(), "medium".into(), "high".into(), "xhigh".into()],
        default_reasoning_mode: Some("high".into()),
    }
}

fn fixture_target(
    scope: &str,
) -> crate::services::reasoning_continuity::contract::ContinuationTarget {
    use crate::services::reasoning_continuity::contract::{
        ContinuationTarget, ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget,
        RouteId,
    };
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::XaiOauth,
        model_id: "grok-4.6".into(),
        credential_scope: CredentialScope::authenticated(scope).unwrap(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

#[test]
fn backend_paths_are_pinned_to_the_subscription_proxy() {
    assert_eq!(
        backend_path(XaiBackend::ChatCompletions),
        "/chat/completions"
    );
    assert_eq!(backend_path(XaiBackend::Responses), "/responses");
    assert!(!crate::services::llm_oauth::XAI_PROXY_BASE_URL.contains("api.x.ai"));
}

#[test]
fn responses_payload_uses_catalog_reasoning_and_never_a_remote_route() {
    let prepared = super::xai_oauth_payload::build_with_evidence(
        &catalog_model(),
        &[ChatMessage::user("bonjour".into())],
        &[],
        Some("xhigh"),
        Some("session-fixture"),
        None,
    )
    .unwrap();
    let payload = prepared.payload;
    assert_eq!(payload["model"], "grok-4.6");
    assert_eq!(payload["reasoning"]["effort"], "xhigh");
    assert_eq!(payload["stream"], true);
    assert!(payload.get("base_url").is_none());
}

#[test]
fn chat_reasoning_is_restricted_by_the_subscription_catalog() {
    let mut model = catalog_model();
    model.reasoning_modes.retain(|mode| mode != "xhigh");

    assert_eq!(catalog_reasoning_mode(&model, Some("low")), Some("low"));
    assert_eq!(catalog_reasoning_mode(&model, Some("xhigh")), Some("high"));
    assert_eq!(catalog_reasoning_mode(&model, None), Some("high"));
}

#[test]
fn chat_request_uses_the_subscription_catalog_restriction() {
    let messages = [];
    let tools = [];
    let request = RequestConfig {
        provider_id: "xai-oauth",
        model: "grok-4.6",
        messages: &messages,
        tools: &tools,
        think: true,
        reasoning_mode: Some("xhigh"),
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("session-fixture"),
        fast_mode: crate::services::llm::fast_mode::FastModeRequest::Unsupported,
        continuation_target: None,
    };
    let mut model = catalog_model();
    model.backend = XaiBackend::ChatCompletions;
    model.reasoning_modes = vec!["low".into(), "high".into()];
    model.default_reasoning_mode = Some("high".into());

    let prepared = prepare_chat_request(request, &model);

    assert_eq!(prepared.reasoning_mode, Some("high"));
}

#[test]
fn required_continuity_cannot_use_the_chat_completions_backend() {
    let messages = [ChatMessage::user("continue".into())];
    let target = fixture_target("xai-oauth-scope");
    let request = RequestConfig {
        provider_id: "xai-oauth",
        model: "grok-4.6",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("high"),
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("session-fixture"),
        fast_mode: crate::services::llm::fast_mode::FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };

    assert_eq!(
        validate_backend(XaiBackend::ChatCompletions, &request),
        Err("reasoning_continuity_invalid".to_string())
    );
    assert_eq!(validate_backend(XaiBackend::Responses, &request), Ok(()));
}

#[test]
fn resource_exhausted_without_retry_after_is_not_a_retryable_rate_limit() {
    assert_eq!(
        classify_status(
            crate::services::llm::route_profile::ErrorPolicy::XaiOauth,
            429,
            r#"{"code":"resource-exhausted"}"#,
            false,
        ),
        "provider_quota_exhausted"
    );
    assert_eq!(
        classify_status(
            crate::services::llm::route_profile::ErrorPolicy::XaiOauth,
            429,
            "{}",
            true,
        ),
        "rate_limit"
    );
    assert_eq!(
        classify_status(
            crate::services::llm::route_profile::ErrorPolicy::XaiOauth,
            401,
            "",
            false,
        ),
        "oauth_reauthentication_required"
    );
    assert_eq!(
        classify_status(
            crate::services::llm::route_profile::ErrorPolicy::XaiOauth,
            403,
            "",
            false,
        ),
        "provider_access_unavailable"
    );
}

#[test]
fn oauth_responses_replays_local_items_without_exposing_a_public_xai_route() {
    let target = fixture_target("xai-oauth-scope");
    let assistant = ChatMessage::assistant(
        "visible".into(),
        None,
        Some(crate::services::reasoning_continuity::envelope::ReasoningEnvelope::new(
            crate::services::reasoning_continuity::contract::ContractId::XaiResponsesV1,
            crate::services::reasoning_continuity::envelope::ReasoningSource::from_target(
                target.replay().unwrap(),
            ),
            crate::services::reasoning_continuity::envelope::CompletionState::Complete,
            crate::services::reasoning_continuity::envelope::ContinuationState::ResponsesLocal {
                items: vec![
                    serde_json::json!({"type":"reasoning","encrypted_content":"opaque"}),
                    serde_json::json!({"type":"message","content":[]}),
                ],
            },
            Vec::new(),
        )),
        None,
        None,
    );
    let prepared = super::xai_oauth_payload::build_with_evidence(
        &catalog_model(),
        &[assistant, ChatMessage::user("continue".into())],
        &[],
        Some("high"),
        Some("session-fixture"),
        Some(&target),
    )
    .unwrap();
    let payload = prepared.payload;

    assert_eq!(payload["input"][0]["type"], "reasoning");
    assert_eq!(payload["input"][1]["type"], "message");
    assert_eq!(payload["input"][2]["role"], "user");
    assert!(payload.get("base_url").is_none());
    assert_eq!(prepared.replayed.len(), 1);
}
