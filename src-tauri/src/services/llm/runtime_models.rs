use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, RwLock};

use super::types::ModelInfo;

const MAX_PROVIDERS: usize = 16;
const MAX_MODELS_PER_PROVIDER: usize = 500;

#[derive(Default)]
struct RuntimeRegistry {
    providers: HashMap<String, HashMap<String, ModelInfo>>,
    recency: VecDeque<String>,
}

static MODELS: LazyLock<RwLock<RuntimeRegistry>> =
    LazyLock::new(|| RwLock::new(RuntimeRegistry::default()));

pub fn replace_provider(provider_id: &str, models: &[ModelInfo]) {
    if !valid_provider_id(provider_id) {
        return;
    }
    let Ok(mut registry) = MODELS.write() else {
        return;
    };
    let mut provider_models = HashMap::with_capacity(models.len().min(MAX_MODELS_PER_PROVIDER));
    for model in models.iter().take(MAX_MODELS_PER_PROVIDER) {
        if valid_model_id(&model.id) {
            provider_models.insert(model.id.clone(), model.clone());
        }
    }
    registry.replace(provider_id, provider_models);
}

pub fn lookup(provider_id: &str, model_id: &str) -> Option<ModelInfo> {
    let registry = MODELS.read().ok()?;
    registry.providers.get(provider_id)?.get(model_id).cloned()
}

impl RuntimeRegistry {
    fn replace(&mut self, provider_id: &str, models: HashMap<String, ModelInfo>) {
        self.recency.retain(|id| id != provider_id);
        if !self.providers.contains_key(provider_id) && self.providers.len() >= MAX_PROVIDERS {
            if let Some(evicted) = self.recency.pop_front() {
                self.providers.remove(&evicted);
            }
        }
        self.providers.insert(provider_id.to_string(), models);
        self.recency.push_back(provider_id.to_string());
    }
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

fn valid_provider_id(provider_id: &str) -> bool {
    !provider_id.is_empty()
        && provider_id.len() <= 32
        && provider_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'-' | b'_'))
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
            max_output_tokens: Some(64_000),
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

    #[test]
    fn catalogs_are_isolated_by_provider() {
        replace_provider("openrouter", &[model("shared".to_string())]);
        replace_provider("openai", &[model("shared".to_string())]);

        assert_eq!(
            lookup("openrouter", "shared").unwrap().max_output_tokens,
            Some(64_000)
        );
        assert!(lookup("unknown", "shared").is_none());
    }

    #[test]
    fn oldest_provider_is_evicted_at_capacity() {
        let mut registry = RuntimeRegistry::default();
        for index in 0..=MAX_PROVIDERS {
            registry.replace(&format!("provider-{index}"), HashMap::new());
        }

        assert_eq!(registry.providers.len(), MAX_PROVIDERS);
        assert!(!registry.providers.contains_key("provider-0"));
        assert!(registry
            .providers
            .contains_key(&format!("provider-{MAX_PROVIDERS}")));
    }
}
