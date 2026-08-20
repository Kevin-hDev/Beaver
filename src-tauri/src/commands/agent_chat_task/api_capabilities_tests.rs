use super::{capability, model_capability_provider_id};
use crate::services::llm::{route, tool_capable};

#[test]
fn local_false_is_authoritative() {
    assert!(!capability(true, false, true, true));
    assert!(capability(true, true, false, false));
    assert!(capability(false, false, true, false));
}

#[test]
fn codex_uses_openai_only_for_model_capabilities() {
    let capability_provider = model_capability_provider_id("codex-oauth");

    assert_eq!(capability_provider, "openai");
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
    assert!(tool_capable::supports_tools(
        capability_provider,
        "gpt-5.6-luna"
    ));
}

#[test]
fn capability_mapping_does_not_change_codex_reasoning_modes() {
    let modes = crate::services::reasoning::supported_modes("codex-oauth", "gpt-5.6-sol", true);

    assert!(modes.contains(&"ultra"));
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
}

#[test]
fn existing_oauth_aliases_keep_their_capability_catalogs() {
    assert_eq!(model_capability_provider_id("xai-oauth"), "xai");
    assert_eq!(model_capability_provider_id("moonshot-oauth"), "moonshot");
    assert_eq!(model_capability_provider_id("mistral"), "mistral");
}
