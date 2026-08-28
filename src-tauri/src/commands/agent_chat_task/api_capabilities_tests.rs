use super::resolve;
use crate::commands::agent_chat_task::StreamCapabilityHints;
use crate::services::llm::route;

#[tokio::test]
async fn embedded_false_is_authoritative_over_the_legacy_name() {
    let capabilities = resolve("openai", "o3-mini", &Default::default()).await;

    assert!(capabilities.tools);
    assert!(capabilities.thinking);
    assert!(!capabilities.vision);
}

#[tokio::test]
async fn codex_without_hints_keeps_tools_for_sampled_model_ids() {
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
    for model in ["gpt-5.6-luna", "gpt-5.3-codex-spark"] {
        assert!(
            resolve("codex-oauth", model, &StreamCapabilityHints::default())
                .await
                .tools
        );
    }
    let canonical = resolve("codex-oauth", "gpt-5.6-luna", &Default::default()).await;
    let forged = resolve(
        "codex-oauth",
        "gpt-5.6-luna",
        &StreamCapabilityHints {
            supports_tools: Some(false),
            supports_thinking: Some(false),
            supports_vision: Some(false),
        },
    )
    .await;
    assert_eq!(forged.tools, canonical.tools);
    assert_eq!(forged.thinking, canonical.thinking);
    assert_eq!(forged.vision, canonical.vision);
}

#[tokio::test]
async fn public_api_capabilities_ignore_contradictory_frontend_hints() {
    let denied = resolve(
        "deepseek",
        "deepseek-chat",
        &StreamCapabilityHints {
            supports_tools: Some(false),
            supports_thinking: Some(false),
            supports_vision: Some(false),
        },
    )
    .await;
    let forced = resolve(
        "deepseek",
        "deepseek-chat",
        &StreamCapabilityHints {
            supports_tools: Some(true),
            supports_thinking: Some(true),
            supports_vision: Some(true),
        },
    )
    .await;
    assert_eq!(denied.tools, forced.tools);
    assert_eq!(denied.thinking, forced.thinking);
    assert_eq!(denied.vision, forced.vision);
}

#[test]
fn capability_mapping_does_not_change_codex_reasoning_modes() {
    let modes = crate::services::reasoning::supported_modes("codex-oauth", "gpt-5.6-sol", true);

    assert!(modes.iter().any(|mode| mode == "ultra"));
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
}
