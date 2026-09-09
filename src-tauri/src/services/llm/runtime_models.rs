use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, RwLock};

use super::types::ModelInfo;

const MAX_RUNTIME_PROVIDERS: usize = 16;

#[derive(Default)]
struct RuntimeRegistry {
    providers: HashMap<String, HashMap<String, ModelInfo>>,
    recency: VecDeque<String>,
}

static MODELS: LazyLock<RwLock<RuntimeRegistry>> =
    LazyLock::new(|| RwLock::new(RuntimeRegistry::default()));

#[cfg(test)]
static TEST_MUTATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
pub(crate) async fn test_mutation_lock() -> tokio::sync::MutexGuard<'static, ()> {
    TEST_MUTATION_LOCK.lock().await
}

pub fn replace_provider(provider_id: &str, models: &[ModelInfo]) -> Result<(), &'static str> {
    if !valid_provider_id(provider_id) {
        return Err("invalid_provider");
    }
    let limit = super::catalog_limits::max_dynamic_models(provider_id);
    if provider_id == "openrouter" && models.len() > limit {
        return Err("too_many_models");
    }
    let mut provider_models = HashMap::with_capacity(models.len().min(limit));
    let mut invalid_entries = 0usize;
    // The larger complete-catalog policy is specific to OpenRouter. Native
    // routes retain their existing bounded prefix rather than gaining new errors.
    for model in models.iter().take(limit) {
        if !valid_model_id(&model.id) {
            invalid_entries += 1;
            continue;
        }
        if super::provider_model_registry_validation::valid_reasoning_contract(
            model.supports_thinking,
            &model.reasoning_modes,
            model.default_reasoning_mode.as_deref(),
        )
        .is_err()
        {
            invalid_entries += 1;
            continue;
        }
        if provider_models
            .insert(model.id.clone(), model.clone())
            .is_some()
            && provider_id == "openrouter"
        {
            return Err("duplicate_model_id");
        }
    }
    if invalid_entries > 0 {
        ::log::warn!("[runtime catalog] ignored invalid entries count={invalid_entries}");
    }
    let Ok(mut registry) = MODELS.write() else {
        return Err("registry_unavailable");
    };
    registry.replace(provider_id, provider_models);
    Ok(())
}

pub fn lookup(provider_id: &str, model_id: &str) -> Option<ModelInfo> {
    let registry = MODELS.read().ok()?;
    registry.providers.get(provider_id)?.get(model_id).cloned()
}

impl RuntimeRegistry {
    fn replace(&mut self, provider_id: &str, models: HashMap<String, ModelInfo>) {
        self.recency.retain(|id| id != provider_id);
        if !self.providers.contains_key(provider_id)
            && self.providers.len() >= MAX_RUNTIME_PROVIDERS
        {
            if let Some(evicted) = self.recency.pop_front() {
                self.providers.remove(&evicted);
            }
        }
        self.providers.insert(provider_id.to_string(), models);
        self.recency.push_back(provider_id.to_string());
    }
}

pub(crate) fn valid_model_id(model_id: &str) -> bool {
    crate::services::model_identifier::is_valid_model_id(model_id)
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
            reasoning_contract: None,
            supports_fast_mode: false,
            reasoning_modes: vec!["auto".to_string()],
            default_reasoning_mode: Some("auto".to_string()),
            context_usage_includes_reasoning: true,
            is_free: true,
        }
    }

    #[tokio::test]
    async fn catalog_rows_preserve_native_runtime_limit_policy() {
        let _guard = test_mutation_lock().await;
        replace_provider("moonshot", &[model("stable".to_string())]).unwrap();
        let models = (0..600)
            .map(|index| model(format!("kimi-{index}")))
            .collect::<Vec<_>>();
        replace_provider("moonshot", &models).unwrap();
        assert!(lookup("moonshot", "kimi-0").is_some());
        assert!(lookup("moonshot", "kimi-499").is_some());
        assert!(lookup("moonshot", "kimi-500").is_none());
        replace_provider(
            "moonshot",
            &[model("../invalid".to_string()), model("stable".to_string())],
        )
        .unwrap();
        assert!(lookup("moonshot", "stable").is_some());
        assert!(lookup("moonshot", "../invalid").is_none());

        let mut invalid_reasoning = model("invalid-reasoning".to_string());
        invalid_reasoning.reasoning_modes = vec!["quantum".to_string()];
        invalid_reasoning.default_reasoning_mode = Some("quantum".to_string());
        replace_provider(
            "moonshot",
            &[invalid_reasoning, model("stable".to_string())],
        )
        .unwrap();
        assert!(lookup("moonshot", "stable").is_some());
        assert!(lookup("moonshot", "invalid-reasoning").is_none());
    }

    #[tokio::test]
    async fn duplicate_ids_are_rejected_without_replacing_the_registry() {
        let _guard = test_mutation_lock().await;
        replace_provider("openrouter", &[model("stable".to_string())]).unwrap();
        let duplicate = model("duplicate".to_string());

        assert_eq!(
            replace_provider("openrouter", &[duplicate.clone(), duplicate]),
            Err("duplicate_model_id")
        );
        assert!(lookup("openrouter", "stable").is_some());
    }

    #[tokio::test]
    async fn openrouter_runtime_limit_rejects_overflow_atomically() {
        let _guard = test_mutation_lock().await;
        let mut models = (0..1_000)
            .map(|index| model(format!("vendor/model-{index}")))
            .collect::<Vec<_>>();
        replace_provider("openrouter", &models).unwrap();
        assert!(lookup("openrouter", "vendor/model-999").is_some());
        models.push(model("vendor/overflow".to_string()));
        assert_eq!(
            replace_provider("openrouter", &models),
            Err("too_many_models")
        );
        assert!(lookup("openrouter", "vendor/model-999").is_some());
        assert!(lookup("openrouter", "vendor/overflow").is_none());
    }

    #[tokio::test]
    async fn catalogs_are_isolated_by_provider() {
        let _guard = test_mutation_lock().await;
        replace_provider("openrouter", &[model("shared".to_string())]).unwrap();
        replace_provider("openai", &[model("shared".to_string())]).unwrap();

        assert_eq!(
            lookup("openrouter", "shared").unwrap().max_output_tokens,
            Some(64_000)
        );
        assert!(lookup("unknown", "shared").is_none());
    }

    #[tokio::test]
    async fn runtime_catalog_accepts_a_routed_model_suffix() {
        let _guard = test_mutation_lock().await;
        let id = "google/gemma-4-31b-it:free";

        replace_provider("openrouter", &[model(id.to_string())]).unwrap();

        assert!(lookup("openrouter", id).is_some());
    }

    #[test]
    fn oldest_provider_is_evicted_at_capacity() {
        let mut registry = RuntimeRegistry::default();
        for index in 0..=MAX_RUNTIME_PROVIDERS {
            registry.replace(&format!("provider-{index}"), HashMap::new());
        }

        assert_eq!(registry.providers.len(), MAX_RUNTIME_PROVIDERS);
        assert!(!registry.providers.contains_key("provider-0"));
        assert!(registry
            .providers
            .contains_key(&format!("provider-{MAX_RUNTIME_PROVIDERS}")));
    }
}
