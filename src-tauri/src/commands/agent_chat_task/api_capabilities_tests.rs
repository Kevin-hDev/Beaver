use super::{capability, resolve};
use crate::commands::agent_chat_task::StreamCapabilityHints;
use crate::services::llm::route;

#[test]
fn local_false_is_authoritative() {
    assert!(!capability(true, false, true, true));
    assert!(capability(true, true, false, false));
    assert!(capability(false, false, true, false));
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
    assert!(
        !resolve(
            "codex-oauth",
            "gpt-5.6-luna",
            &StreamCapabilityHints {
                supports_tools: Some(false),
                ..Default::default()
            },
        )
        .await
        .tools
    );
}

#[test]
fn capability_mapping_does_not_change_codex_reasoning_modes() {
    let modes = crate::services::reasoning::supported_modes("codex-oauth", "gpt-5.6-sol", true);

    assert!(modes.iter().any(|mode| mode == "ultra"));
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
}
