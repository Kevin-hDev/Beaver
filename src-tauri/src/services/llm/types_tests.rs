use super::ModelInfo;

#[test]
fn unknown_reasoning_crosses_the_model_info_serialization_boundary_once() {
    let model = ModelInfo {
        id: "grok-4.6".to_string(),
        display_name: None,
        owned_by: None,
        context_length: None,
        max_output_tokens: None,
        supported_parameters: None,
        catalog_capabilities: Default::default(),
        supports_tools: false,
        supports_vision: false,
        supports_thinking: true,
        reasoning_contract:
            crate::services::llm::model_reasoning_contract::ModelReasoningContract::from_names(
                &[],
                None,
            ),
        supports_fast_mode: false,
        context_usage_includes_reasoning: true,
        is_free: false,
    };

    let serialized = serde_json::to_value(model).expect("serializable ModelInfo");

    assert_eq!(
        serialized["reasoning_contract"]["control"]["kind"],
        "unknown"
    );
    assert!(serialized.get("reasoning_modes").is_none());
    assert!(serialized.get("default_reasoning_mode").is_none());
}
