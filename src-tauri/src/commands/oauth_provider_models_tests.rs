use super::*;

#[test]
fn oauth_model_transports_public_metadata_without_recalculating_it() {
    let model = oauth_model(
        ProviderId::OpenAi,
        ModelInfo {
            id: "gpt-5.6-sol".to_string(),
            display_name: None,
            owned_by: Some("openai".to_string()),
            context_length: Some(258_400),
            max_output_tokens: None,
            supported_parameters: None,
            catalog_capabilities: Default::default(),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            reasoning_contract: Some(
                crate::services::llm::model_reasoning_contract::ModelReasoningContract {
                    mandatory: Some(true),
                    default_enabled: Some(true),
                    supports_max_tokens: None,
                    default_effort: Some(
                        crate::services::reasoning_continuity::contract::ReasoningModeId::High,
                    ),
                    control: crate::services::llm::model_reasoning_contract::ReasoningControl::Efforts(
                        vec![crate::services::reasoning_continuity::contract::ReasoningModeId::High],
                    ),
                },
            ),
            supports_fast_mode: true,
            reasoning_modes: vec!["high".to_string()],
            default_reasoning_mode: Some("high".to_string()),
            context_usage_includes_reasoning: true,
            is_free: false,
        },
    );

    assert!(model.supports_fast_mode);
    assert_eq!(model.connection_id, "codex-oauth");
    assert_eq!(model.provider_display_name, "OpenAI");
    assert!(!model.context_usage_includes_reasoning);
    assert_eq!(
        model
            .reasoning_contract
            .as_ref()
            .and_then(|contract| contract.default_effort),
        Some(crate::services::reasoning_continuity::contract::ReasoningModeId::High)
    );
}
