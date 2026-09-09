use super::request_purpose::RequestPurpose;
use super::stream_dispatch::{
    is_available, resolve_transport_for_test, ClientKind, InvocationKind, RouteSelectionError,
};
use crate::services::llm::route_profile::FragmentMode;
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};
use crate::services::provider_usage::UsageApiFormat;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn xai_model(backend: XaiBackend) -> XaiCatalogModel {
    XaiCatalogModel {
        id: "grok-fixture".into(),
        display_name: "Grok fixture".into(),
        backend,
        context_window: 128_000,
        max_output_tokens: None,
        reasoning_modes: vec![],
        default_reasoning_mode: None,
    }
}

fn openrouter_fixture_body() -> serde_json::Value {
    serde_json::json!({"data":[{
        "id":"moonshotai/kimi-k2.5",
        "architecture":{"input_modalities":["text"],"output_modalities":["text"]},
        "supported_parameters":["reasoning"],
        "reasoning":{
            "mandatory":false,
            "default_enabled":false,
            "supported_efforts":["medium"],
            "default_effort":"medium"
        }
    }]})
}

#[tokio::test]
async fn openrouter_common_and_silent_transport_keep_the_final_catalog_object() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(openrouter_fixture_body()),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/models", server.uri());
    let (loading, silent) = tokio::join!(
        super::openrouter_catalog::list_models_from_url_for_test(&url),
        super::stream_dispatch::resolve_transport(
            "openrouter",
            "moonshotai/kimi-k2.5",
            InvocationKind::Silent,
            RequestPurpose::Automation,
        )
    );
    loading.unwrap();
    assert_eq!(silent.unwrap().client, ClientKind::ChatCompletions);
    assert_eq!(
        super::stream_dispatch::resolve_transport(
            "openrouter",
            "moonshotai/kimi-k2.5",
            InvocationKind::Interactive,
            RequestPurpose::ManualChat,
        )
        .await
        .unwrap()
        .client,
        ClientKind::ChatCompletions
    );
    let model = super::runtime_models::lookup("openrouter", "moonshotai/kimi-k2.5")
        .expect("final OpenRouter catalog model");
    assert!(!model.supports_tools);
    assert!(!model.supports_vision);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("off"));
    assert_eq!(
        model
            .reasoning_contract
            .as_ref()
            .and_then(|contract| contract.default_effort),
        Some(ReasoningModeId::Medium)
    );
    assert_eq!(
        model.supported_parameters.as_deref().unwrap(),
        ["reasoning"]
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn openrouter_debug_fixture_transport_keeps_the_final_catalog_object() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(openrouter_fixture_body()),
        )
        .expect(1)
        .mount(&server)
        .await;
    let fixture = ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::OpenRouter,
        model_id: "moonshotai/kimi-k2.5".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    });

    let url = format!("{}/models", server.uri());
    let (loading, resolved) = tokio::join!(
        super::openrouter_catalog::list_models_from_url_for_test(&url),
        super::stream_dispatch::resolve_fixture_transport(
            "openrouter",
            "moonshotai/kimi-k2.5",
            &fixture,
            RequestPurpose::ManualChat,
        )
    );

    loading.unwrap();
    assert_eq!(resolved.unwrap().client, ClientKind::ChatCompletions);
    let model = super::runtime_models::lookup("openrouter", "moonshotai/kimi-k2.5")
        .expect("final OpenRouter catalog model");
    assert!(!model.supports_tools);
    assert!(!model.supports_vision);
    assert_eq!(model.default_reasoning_mode.as_deref(), Some("off"));
    assert_eq!(
        model
            .reasoning_contract
            .as_ref()
            .and_then(|contract| contract.default_effort),
        Some(ReasoningModeId::Medium)
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn codex_fixture_candidate_reaches_its_bounded_transport() {
    let fixture = ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::CodexOauth,
        model_id: "gpt-6-astra".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    });
    let resolved = super::stream_dispatch::resolve_fixture_transport(
        "codex-oauth",
        "gpt-6-astra",
        &fixture,
        RequestPurpose::ManualChat,
    )
    .await
    .unwrap();
    assert_eq!(resolved.client, ClientKind::Codex);
    for (route, model, purpose) in [
        ("codex-oauth", "other-model", RequestPurpose::ManualChat),
        ("xai-oauth", "gpt-6-astra", RequestPurpose::ManualChat),
        ("codex-oauth", "gpt-6-astra", RequestPurpose::Automation),
    ] {
        assert!(
            super::stream_dispatch::resolve_fixture_transport(route, model, &fixture, purpose)
                .await
                .is_err()
        );
    }
}

#[test]
fn stream_dispatch_and_stream_metrics_select_clients_and_formats_once() {
    let rows = [
        ("codex-oauth", ClientKind::Codex, UsageApiFormat::Responses),
        ("openai", ClientKind::Responses, UsageApiFormat::Responses),
        ("xai", ClientKind::Responses, UsageApiFormat::Responses),
        (
            "google",
            ClientKind::ChatCompletions,
            UsageApiFormat::ChatCompletions,
        ),
        (
            "moonshot",
            ClientKind::ChatCompletions,
            UsageApiFormat::ChatCompletions,
        ),
        (
            "ollama",
            ClientKind::OllamaLocal,
            UsageApiFormat::ChatCompletions,
        ),
    ];
    for (route, client, usage) in rows {
        let resolved = resolve_transport_for_test(
            route,
            InvocationKind::Interactive,
            RequestPurpose::ManualChat,
            None,
        )
        .unwrap();
        assert_eq!(resolved.client, client, "{route}");
        assert_eq!(resolved.usage_api_format, usage, "{route}");
    }
}

#[test]
fn stream_dispatch_uses_the_validated_xai_oauth_backend() {
    for (backend, usage, fragments) in [
        (
            XaiBackend::ChatCompletions,
            UsageApiFormat::ChatCompletions,
            FragmentMode::DifferentialFragments,
        ),
        (
            XaiBackend::Responses,
            UsageApiFormat::Responses,
            FragmentMode::SemanticEvents,
        ),
    ] {
        let resolved = resolve_transport_for_test(
            "xai-oauth",
            InvocationKind::Interactive,
            RequestPurpose::ManualChat,
            Some(xai_model(backend)),
        )
        .unwrap();
        assert_eq!(resolved.client, ClientKind::XaiOauth(backend));
        assert_eq!(resolved.usage_api_format, usage);
        assert_eq!(resolved.fragment_mode, fragments);
        assert_eq!(
            resolved.xai_catalog_model.as_ref().unwrap().backend,
            backend
        );
    }
}

#[test]
fn stream_dispatch_refuses_unavailable_or_unknown_routes_before_payload() {
    for route in ["xai-oauth", "moonshot-oauth"] {
        assert!(!is_available(
            route,
            InvocationKind::Silent,
            RequestPurpose::ManualChat
        ));
        assert!(!is_available(
            route,
            InvocationKind::Interactive,
            RequestPurpose::Automation
        ));
        assert!(!is_available(
            route,
            InvocationKind::Interactive,
            RequestPurpose::ExternalChannel
        ));
    }
    assert_eq!(
        resolve_transport_for_test(
            "unknown",
            InvocationKind::Interactive,
            RequestPurpose::ManualChat,
            None,
        )
        .unwrap_err(),
        RouteSelectionError::UnknownRoute
    );
    assert_eq!(
        resolve_transport_for_test(
            "openai",
            InvocationKind::Interactive,
            RequestPurpose::Unknown,
            None,
        )
        .unwrap_err(),
        RouteSelectionError::Unavailable
    );
}

#[tokio::test]
async fn anthropic_live_route_supports_every_declared_invocation_kind() {
    assert_eq!(
        super::stream_dispatch::resolve_client_for_test("anthropic").unwrap(),
        ClientKind::Anthropic
    );
    for purpose in [
        RequestPurpose::ManualChat,
        RequestPurpose::Automation,
        RequestPurpose::ExternalChannel,
        RequestPurpose::AccountMetadata,
    ] {
        assert_eq!(
            super::stream_dispatch::resolve_transport(
                "anthropic",
                "claude-haiku-4-5-20251001",
                InvocationKind::Interactive,
                purpose,
            )
            .await
            .unwrap()
            .client,
            ClientKind::Anthropic
        );
    }
    assert_eq!(
        super::stream_dispatch::resolve_transport(
            "anthropic",
            "claude-haiku-4-5-20251001",
            InvocationKind::Silent,
            RequestPurpose::ManualChat,
        )
        .await
        .unwrap()
        .client,
        ClientKind::Anthropic
    );

    let fixture = ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Anthropic,
        model_id: "claude-haiku-4-5-20251001".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use: ContinuationUse::UserContinuation,
    });
    assert!(super::stream_dispatch::resolve_fixture_transport(
        "anthropic",
        "claude-haiku-4-5-20251001",
        &fixture,
        RequestPurpose::ManualChat,
    )
    .await
    .is_err());
    assert!(super::stream_dispatch::resolve_fixture_transport(
        "anthropic",
        "claude-haiku-4-5-20251001",
        &fixture,
        RequestPurpose::Automation,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn qwen_live_route_supports_every_declared_invocation_kind() {
    assert_eq!(
        super::stream_dispatch::resolve_client_for_test("qwen").unwrap(),
        ClientKind::ChatCompletions
    );
    for purpose in [
        RequestPurpose::ManualChat,
        RequestPurpose::Automation,
        RequestPurpose::ExternalChannel,
        RequestPurpose::AccountMetadata,
    ] {
        assert_eq!(
            super::stream_dispatch::resolve_transport(
                "qwen",
                "qwen3.8-flash",
                InvocationKind::Interactive,
                purpose,
            )
            .await
            .unwrap()
            .client,
            ClientKind::ChatCompletions
        );
    }
    assert_eq!(
        super::stream_dispatch::resolve_transport(
            "qwen",
            "qwen3.8-flash",
            InvocationKind::Silent,
            RequestPurpose::ManualChat,
        )
        .await
        .unwrap()
        .client,
        ClientKind::ChatCompletions
    );

    let fixture = ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Qwen,
        model_id: "qwen3.8-flash".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Xhigh,
        continuation_use: ContinuationUse::UserContinuation,
    });
    let resolved = super::stream_dispatch::resolve_fixture_transport(
        "qwen",
        "qwen3.8-flash",
        &fixture,
        RequestPurpose::ManualChat,
    )
    .await
    .unwrap();
    assert_eq!(resolved.client, ClientKind::ChatCompletions);

    assert!(super::stream_dispatch::resolve_fixture_transport(
        "qwen",
        "another-model",
        &fixture,
        RequestPurpose::ManualChat,
    )
    .await
    .is_err());
    assert!(super::stream_dispatch::resolve_fixture_transport(
        "qwen",
        "qwen3.8-flash",
        &fixture,
        RequestPurpose::Automation,
    )
    .await
    .is_err());
}
