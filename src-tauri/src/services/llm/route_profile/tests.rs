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
    assert_eq!(profiles.len(), 9);
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
fn route_profile_declares_anthropic_wire_without_activating_a_route() {
    assert_eq!(
        policies::ANTHROPIC_WIRE_TEST.family,
        WireFamily::AnthropicMessages
    );
    assert_eq!(
        policies::ANTHROPIC_WIRE_TEST.tool_results,
        ToolResultPlacement::UserToolResultBlock
    );
    assert!(find("anthropic").is_none());
}

#[test]
fn context_usage_reasoning_policy_is_owned_by_the_route_profile() {
    assert!(!find("codex-oauth")
        .unwrap()
        .context_usage_includes_reasoning());
    assert!(find("openai").unwrap().context_usage_includes_reasoning());
    assert!(find("ollama").unwrap().context_usage_includes_reasoning());
}
