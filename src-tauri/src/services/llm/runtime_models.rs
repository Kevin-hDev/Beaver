use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, RwLock};

use super::types::ModelInfo;

#[path = "runtime_models_snapshot.rs"]
mod snapshot;
pub(crate) use snapshot::snapshot;

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
        let contract_is_valid = match (model.supports_thinking, &model.reasoning_contract) {
            (false, None) => true,
            (true, Some(contract)) => contract.is_valid(),
            _ => false,
        };
        if !contract_is_valid {
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
#[path = "runtime_models_tests.rs"]
mod tests;
