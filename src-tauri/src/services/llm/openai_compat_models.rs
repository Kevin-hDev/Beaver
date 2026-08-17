use super::types::ModelInfo;

pub(super) fn has_static_models(provider_id: &str) -> bool {
    provider_id == "zai"
}

pub(super) fn static_model_infos(provider_id: &str) -> Option<Vec<ModelInfo>> {
    has_static_models(provider_id).then(|| {
        super::provider_model_registry::list(provider_id)
            .into_iter()
            .map(|model| to_model_info(provider_id, model))
            .collect()
    })
}

pub(super) fn ping_model(provider_id: &str) -> &'static str {
    match provider_id {
        "zai" => "glm-4.5-flash",
        _ => "test",
    }
}

fn to_model_info(
    provider_id: &str,
    model: super::provider_model_registry::ProviderModelConfig,
) -> ModelInfo {
    let reasoning_modes = crate::services::reasoning::supported_modes(
        provider_id,
        &model.id,
        model.supports_thinking,
    )
    .iter()
    .map(|mode| mode.to_string())
    .collect();
    ModelInfo {
        id: model.id,
        display_name: None,
        owned_by: None,
        context_length: Some(model.context_window),
        max_output_tokens: model.max_output_tokens,
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        reasoning_modes,
        default_reasoning_mode: None,
        is_free: model.is_free,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xai_uses_its_official_dynamic_endpoint() {
        assert!(!has_static_models("xai"));
        assert!(static_model_infos("xai").is_none());
    }

    #[test]
    fn zai_static_models_expose_reasoning_capabilities() {
        let models = static_model_infos("zai").unwrap();
        let glm_52 = models.iter().find(|m| m.id == "glm-5.2").unwrap();
        let glm_5 = models.iter().find(|m| m.id == "glm-5").unwrap();
        let glm_46 = models.iter().find(|m| m.id == "glm-4.6").unwrap();
        let glm_flash = models.iter().find(|m| m.id == "glm-4.5-flash").unwrap();
        let glm_47_flash = models.iter().find(|m| m.id == "glm-4.7-flash").unwrap();
        let glm_flashx = models.iter().find(|m| m.id == "glm-4.7-flashx").unwrap();
        let glm_vision_flash = models.iter().find(|m| m.id == "glm-4.6v-flash").unwrap();
        let glm_vision_flashx = models.iter().find(|m| m.id == "glm-4.6v-flashx").unwrap();

        assert_eq!(models.len(), 19);
        assert_eq!(glm_47_flash.context_length, Some(200_000));
        assert_eq!(glm_52.context_length, Some(1_000_000));
        assert!(glm_52.supports_thinking);
        assert!(glm_5.supports_thinking);
        assert!(glm_46.supports_thinking);
        assert!(glm_flash.supports_thinking);
        assert!(glm_flash.is_free);
        assert!(glm_47_flash.is_free);
        assert!(glm_vision_flash.is_free);
        assert!(!glm_flashx.is_free);
        assert!(!glm_vision_flashx.is_free);
    }
}
