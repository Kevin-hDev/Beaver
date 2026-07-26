use super::types::ModelInfo;

struct StaticModel {
    id: &'static str,
    ctx: u32,
}

const ZAI_MODELS: &[StaticModel] = &[
    StaticModel {
        id: "glm-5.2",
        ctx: 1_000_000,
    },
    StaticModel {
        id: "glm-5.1",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-5",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-5-code",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.7",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.6",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.5",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5v",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5-air",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5-flash",
        ctx: 128_000,
    },
];

const XAI_MODELS: &[StaticModel] = &[
    StaticModel {
        id: "grok-4.5",
        ctx: 500_000,
    },
    StaticModel {
        id: "grok-4.3",
        ctx: 1_000_000,
    },
    StaticModel {
        id: "grok-4.20-0309-reasoning",
        ctx: 1_000_000,
    },
    StaticModel {
        id: "grok-4.20-0309-non-reasoning",
        ctx: 1_000_000,
    },
    StaticModel {
        id: "grok-build-0.1",
        ctx: 256_000,
    },
];

pub(super) fn has_static_models(provider_id: &str) -> bool {
    static_models(provider_id).is_some()
}

pub(super) fn static_model_infos(provider_id: &str) -> Option<Vec<ModelInfo>> {
    static_models(provider_id).map(|models| {
        models
            .iter()
            .map(|m| {
                let supports_thinking = super::tool_capable::supports_thinking(provider_id, m.id);
                ModelInfo {
                    id: m.id.to_string(),
                    display_name: None,
                    owned_by: None,
                    context_length: Some(m.ctx),
                    supports_tools: super::tool_capable::supports_tools(provider_id, m.id),
                    supports_vision: super::tool_capable::supports_vision(provider_id, m.id),
                    supports_thinking,
                    reasoning_modes: crate::services::reasoning::supported_modes(
                        provider_id,
                        m.id,
                        supports_thinking,
                    )
                    .iter()
                    .map(|mode| mode.to_string())
                    .collect(),
                    default_reasoning_mode: None,
                    is_free: false,
                }
            })
            .collect()
    })
}

pub(super) fn ping_model(provider_id: &str) -> &'static str {
    match provider_id {
        "zai" => "glm-4.5-flash",
        "xai" => "grok-4.3",
        _ => "test",
    }
}

fn static_models(provider_id: &str) -> Option<&'static [StaticModel]> {
    match provider_id {
        "zai" => Some(ZAI_MODELS),
        "xai" => Some(XAI_MODELS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xai_static_models_expose_reasoning_capabilities() {
        let models = static_model_infos("xai").unwrap();
        let ids = models
            .iter()
            .map(|model| model.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            [
                "grok-4.5",
                "grok-4.3",
                "grok-4.20-0309-reasoning",
                "grok-4.20-0309-non-reasoning",
                "grok-build-0.1",
            ]
        );
        assert_eq!(models[0].context_length, Some(500_000));
        assert_eq!(models[1].context_length, Some(1_000_000));
        assert_eq!(models[4].context_length, Some(256_000));
        assert_eq!(models[0].reasoning_modes, ["low", "medium", "high"]);
        assert_eq!(models[1].reasoning_modes, ["off", "low", "medium", "high"]);
        assert_eq!(models[2].reasoning_modes, ["auto"]);
        assert!(!models[3].supports_thinking);
        assert!(models[3].reasoning_modes.is_empty());
        assert_eq!(models[4].reasoning_modes, ["auto"]);
        assert_eq!(ping_model("xai"), "grok-4.3");
    }

    #[test]
    fn zai_static_models_expose_reasoning_capabilities() {
        let models = static_model_infos("zai").unwrap();
        let glm_52 = models.iter().find(|m| m.id == "glm-5.2").unwrap();
        let glm_5 = models.iter().find(|m| m.id == "glm-5").unwrap();
        let glm_46 = models.iter().find(|m| m.id == "glm-4.6").unwrap();
        let glm_flash = models.iter().find(|m| m.id == "glm-4.5-flash").unwrap();

        assert_eq!(glm_52.context_length, Some(1_000_000));
        assert!(glm_52.supports_thinking);
        assert!(glm_5.supports_thinking);
        assert!(glm_46.supports_thinking);
        assert!(glm_flash.supports_thinking);
    }
}
