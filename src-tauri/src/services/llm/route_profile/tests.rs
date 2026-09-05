use super::*;
use crate::services::reasoning_continuity::contract::RouteId;

#[test]
fn route_profile_inventory_is_complete_and_unique() {
    let profiles = all().collect::<Vec<_>>();
    assert_eq!(profiles.len(), RouteId::ALL.len());
    for id in RouteId::ALL {
        assert_eq!(
            profiles.iter().filter(|profile| profile.id == id).count(),
            1,
            "{id:?}"
        );
        assert!(find_id(id).is_some(), "{id:?}");
    }
}

#[test]
fn route_profile_public_catalog_contains_only_api_key_routes() {
    let profiles = public_api().collect::<Vec<_>>();
    assert_eq!(profiles.len(), 11);
    assert!(profiles
        .iter()
        .all(|profile| matches!(profile.catalog, CatalogPolicy::PublicApi { .. })));
    assert!(profiles
        .iter()
        .all(|profile| matches!(profile.auth, AuthKind::ApiKey { .. })));
}

#[test]
fn route_profile_keeps_oauth_origins_and_local_endpoint_distinct() {
    assert!(matches!(
        find("xai-oauth").unwrap().auth,
        AuthKind::OAuth {
            provider: crate::services::llm_oauth::LlmOAuthProvider::Xai,
            ..
        }
    ));
    assert!(matches!(
        find("moonshot-oauth").unwrap().auth,
        AuthKind::OAuth {
            provider: crate::services::llm_oauth::LlmOAuthProvider::Kimi,
            ..
        }
    ));
    let ollama = find("ollama").unwrap();
    assert_eq!(ollama.client, ClientSelector::OllamaLocal);
    assert_eq!(ollama.endpoint, EndpointPolicy::OllamaLocal);
}

#[test]
fn anthropic_profile_is_native_complete_and_public() {
    let profile = find("anthropic").expect("anthropic profile");
    assert_eq!(profile.id, RouteId::Anthropic);
    assert_eq!(profile.client, ClientSelector::Anthropic);
    assert_eq!(profile.wire.family, WireFamily::AnthropicMessages);
    assert_eq!(profile.wire.fragments, FragmentMode::SemanticEvents);
    assert_eq!(
        profile.wire.tool_results,
        ToolResultPlacement::UserToolResultBlock
    );
    assert_eq!(profile.wire.images, ImageFormat::AnthropicBlock);
    assert_eq!(profile.availability, policies::AVAILABLE_ANY);
    assert!(matches!(profile.catalog, CatalogPolicy::PublicApi { .. }));
    assert!(has_dynamic_reasoning_catalog("anthropic"));
    assert!(matches!(
        profile.auth,
        AuthKind::ApiKey {
            header: ApiKeyHeader::XApiKey,
            ..
        }
    ));
    let AuthKind::ApiKey { headers, .. } = profile.auth else {
        panic!("Anthropic API key auth")
    };
    assert_eq!(headers, &[("anthropic-version", "2023-06-01")]);
    assert!(super::catalog::configurable().any(|candidate| candidate.id == RouteId::Anthropic));
    assert!(public_api().any(|candidate| candidate.id == RouteId::Anthropic));
}

#[test]
fn qwen_profile_is_public_and_uses_its_configured_endpoint() {
    let profile = find("qwen").expect("qwen profile");
    assert_eq!(profile.id, RouteId::Qwen);
    assert_eq!(profile.client, ClientSelector::OpenAiCompat);
    assert_eq!(profile.wire.family, WireFamily::OpenAiChatCompletions);
    assert_eq!(profile.wire.images, ImageFormat::OpenAiNested);
    assert_eq!(profile.availability, policies::AVAILABLE_ANY);
    assert!(matches!(
        profile.endpoint,
        EndpointPolicy::ProviderConnection {
            resolver: ConnectionEndpointResolver::QwenModelStudio
        }
    ));
    assert!(matches!(profile.catalog, CatalogPolicy::PublicApi { .. }));
    assert!(has_dynamic_reasoning_catalog("qwen"));
    assert!(!has_dynamic_reasoning_catalog("openai"));
    assert!(super::catalog::configurable().any(|candidate| candidate.id == RouteId::Qwen));
    assert!(public_api().any(|candidate| candidate.id == RouteId::Qwen));
}

#[test]
fn context_usage_reasoning_policy_is_owned_by_the_route_profile() {
    assert!(!find("codex-oauth")
        .unwrap()
        .context_usage_includes_reasoning());
    assert!(find("openai").unwrap().context_usage_includes_reasoning());
    assert!(find("ollama").unwrap().context_usage_includes_reasoning());
}

#[test]
fn xai_and_codex_profiles_explicitly_refuse_tool_result_media() {
    for provider in ["xai", "xai-oauth", "codex-oauth"] {
        assert_eq!(
            find(provider).expect("profile").wire.tool_result_media,
            ToolResultMedia::TextOnly,
            "{provider}"
        );
    }
}

#[test]
fn xai_oauth_chat_catalog_entries_have_their_own_text_only_wire() {
    let policy = super::xai_oauth_chat_payload_policy("grok-code-fast-1").expect("policy");
    assert_eq!(policy.tool_result_media, ToolResultMedia::TextOnly);
    assert_eq!(policy.message.tool_results, ToolResultPlacement::ToolRole);
    assert_eq!(policy.message.images, ImageFormat::OpenAiNested);
}
