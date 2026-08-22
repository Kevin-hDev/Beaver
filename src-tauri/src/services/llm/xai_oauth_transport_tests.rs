use super::xai_oauth_transport::{backend_path, build_responses_payload, classify_status};
use crate::services::agent_local::types_ollama::ChatMessage;
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
    let payload = build_responses_payload(
        &catalog_model(),
        &[ChatMessage {
            role: "user".into(),
            content: "bonjour".into(),
            ..Default::default()
        }],
        &[],
        Some("xhigh"),
        Some("session-fixture"),
    );
    assert_eq!(payload["model"], "grok-4.6");
    assert_eq!(payload["reasoning"]["effort"], "xhigh");
    assert_eq!(payload["stream"], true);
    assert!(payload.get("base_url").is_none());
}

#[test]
fn resource_exhausted_without_retry_after_is_not_a_retryable_rate_limit() {
    assert_eq!(
        classify_status(429, r#"{"code":"resource-exhausted"}"#, false),
        "provider_quota_exhausted"
    );
    assert_eq!(classify_status(429, "{}", true), "rate_limit");
    assert_eq!(
        classify_status(401, "", false),
        "oauth_reauthentication_required"
    );
    assert_eq!(
        classify_status(403, "", false),
        "provider_access_unavailable"
    );
}
