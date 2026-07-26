use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use super::types::ModelInfo;

const MAX_MODELS: usize = 500;
static MODELS: LazyLock<RwLock<HashMap<String, ModelInfo>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn replace_provider(provider_id: &str, models: &[ModelInfo]) {
    if !matches!(provider_id, "xai" | "moonshot") {
        return;
    }
    let Ok(mut registry) = MODELS.write() else {
        return;
    };
    let prefix = format!("{provider_id}/");
    registry.retain(|key, _| !key.starts_with(&prefix));
    for model in models.iter().take(MAX_MODELS) {
        if valid_model_id(&model.id) && registry.len() < MAX_MODELS {
            registry.insert(format!("{provider_id}/{}", model.id), model.clone());
        }
    }
}

pub fn lookup(provider_id: &str, model_id: &str) -> Option<ModelInfo> {
    let registry = MODELS.read().ok()?;
    registry.get(&format!("{provider_id}/{model_id}")).cloned()
}

pub(crate) fn valid_model_id(model_id: &str) -> bool {
    !model_id.is_empty()
        && model_id.len() <= 128
        && !model_id.contains("..")
        && !model_id.starts_with('/')
        && model_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: String) -> ModelInfo {
        ModelInfo {
            id,
            display_name: None,
            owned_by: None,
            context_length: Some(256_000),
            supports_tools: true,
            supports_vision: true,
            supports_thinking: true,
            reasoning_modes: vec!["auto".to_string()],
            default_reasoning_mode: Some("auto".to_string()),
            is_free: true,
        }
    }

    #[test]
    fn runtime_catalog_is_bounded_and_validated() {
        let models = (0..600)
            .map(|index| model(format!("kimi-{index}")))
            .collect::<Vec<_>>();
        replace_provider("moonshot", &models);
        assert!(lookup("moonshot", "kimi-0").is_some());
        assert!(lookup("moonshot", "kimi-499").is_some());
        assert!(lookup("moonshot", "kimi-500").is_none());
        replace_provider("moonshot", &[model("../invalid".to_string())]);
        assert!(lookup("moonshot", "../invalid").is_none());
    }
}
