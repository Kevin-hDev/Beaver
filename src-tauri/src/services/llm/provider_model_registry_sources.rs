pub(super) struct EmbeddedProviderModels {
    pub provider_id: &'static str,
    pub json: &'static str,
}

pub(super) const SOURCES: &[EmbeddedProviderModels] = &[
    EmbeddedProviderModels {
        provider_id: "groq",
        json: include_str!("../../../resources/provider-models/groq.json"),
    },
    EmbeddedProviderModels {
        provider_id: "google",
        json: include_str!("../../../resources/provider-models/google.json"),
    },
    EmbeddedProviderModels {
        provider_id: "mistral",
        json: include_str!("../../../resources/provider-models/mistral.json"),
    },
    EmbeddedProviderModels {
        provider_id: "cerebras",
        json: include_str!("../../../resources/provider-models/cerebras.json"),
    },
    EmbeddedProviderModels {
        provider_id: "openrouter",
        json: include_str!("../../../resources/provider-models/openrouter.json"),
    },
    EmbeddedProviderModels {
        provider_id: "openai",
        json: include_str!("../../../resources/provider-models/openai.json"),
    },
    EmbeddedProviderModels {
        provider_id: "deepseek",
        json: include_str!("../../../resources/provider-models/deepseek.json"),
    },
    EmbeddedProviderModels {
        provider_id: "xai",
        json: include_str!("../../../resources/provider-models/xai.json"),
    },
    EmbeddedProviderModels {
        provider_id: "moonshot",
        json: include_str!("../../../resources/provider-models/moonshot.json"),
    },
    EmbeddedProviderModels {
        provider_id: "zai",
        json: include_str!("../../../resources/provider-models/zai.json"),
    },
];
