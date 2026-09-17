use super::resolve;
use crate::services::llm::route;

#[tokio::test]
async fn embedded_false_is_authoritative_over_the_legacy_name() {
    let capabilities = resolve("openai", "o3-mini").await;

    assert!(capabilities.tools);
    assert!(capabilities.thinking);
    assert!(!capabilities.vision);
}

#[tokio::test]
async fn codex_catalog_keeps_tools_for_sampled_model_ids() {
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
    for model in ["gpt-5.6-luna", "gpt-5.3-codex-spark"] {
        assert!(resolve("codex-oauth", model).await.tools);
    }
}

#[test]
fn capability_mapping_does_not_change_codex_reasoning_modes() {
    let modes = crate::services::reasoning::supported_modes("codex-oauth", "gpt-5.6-sol", true);

    assert!(modes.iter().any(|mode| mode == "ultra"));
    assert_eq!(route::canonical_provider_id("codex-oauth"), "codex-oauth");
}
