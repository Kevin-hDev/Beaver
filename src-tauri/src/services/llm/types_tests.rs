use super::ModelInfo;

#[test]
fn empty_reasoning_modes_cross_the_model_info_serialization_boundary() {
    let model = ModelInfo {
        id: "grok-4.6".to_string(),
        display_name: None,
        owned_by: None,
        context_length: None,
        max_output_tokens: None,
        supports_tools: false,
        supports_vision: false,
        supports_thinking: true,
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        context_usage_includes_reasoning: true,
        is_free: false,
    };

    let serialized = serde_json::to_value(model).expect("serializable ModelInfo");

    assert_eq!(serialized["reasoning_modes"], serde_json::json!([]));
}
