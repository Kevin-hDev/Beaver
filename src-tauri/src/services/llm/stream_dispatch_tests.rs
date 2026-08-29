use super::request_purpose::RequestPurpose;
use super::stream_dispatch::{
    is_available, resolve_transport_for_test, ClientKind, InvocationKind, RouteSelectionError,
};
use crate::services::llm::route_profile::FragmentMode;
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};
use crate::services::provider_usage::UsageApiFormat;

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
