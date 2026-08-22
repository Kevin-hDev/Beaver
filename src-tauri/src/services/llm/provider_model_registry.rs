use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::provider_model_registry_sources::{EmbeddedProviderModels, SOURCES};
use super::provider_model_registry_validation::{
    valid_date, valid_provider_id, valid_reasoning_contract, valid_source_url,
};

const MAX_PROVIDERS: usize = 16;
const MAX_MODELS_PER_PROVIDER: usize = 500;
const MAX_ALIASES_PER_MODEL: usize = 32;
const MAX_SOURCE_URLS: usize = 16;
const MAX_CONTEXT_TOKENS: u32 = 4_000_000;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderModelConfig {
    pub id: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Vrai uniquement lorsqu'une source tarifaire officielle confirme un coût nul.
    #[serde(default)]
    pub is_free: bool,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub default_output_tokens: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    #[serde(default)]
    pub supports_fast_mode: bool,
    #[serde(default)]
    pub reasoning_modes: Vec<String>,
    #[serde(default)]
    pub default_reasoning_mode: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderModelFile {
    provider: String,
    schema_version: u8,
    verified_at: String,
    source_urls: Vec<String>,
    #[serde(default)]
    inherits_upstream: bool,
    models: Vec<ProviderModelConfig>,
}

struct ProviderModels {
    ordered: Vec<ProviderModelConfig>,
    by_id: HashMap<String, ProviderModelConfig>,
    inherits_upstream: bool,
}

pub(crate) struct ProviderModelRegistry {
    providers: HashMap<String, ProviderModels>,
}

static REGISTRY: OnceLock<Option<ProviderModelRegistry>> = OnceLock::new();

pub fn lookup(provider_id: &str, model_id: &str) -> Option<ProviderModelConfig> {
    registry()?
        .providers
        .get(provider_id)?
        .by_id
        .get(model_id)
        .cloned()
}

pub fn list(provider_id: &str) -> Vec<ProviderModelConfig> {
    registry()
        .and_then(|registry| registry.providers.get(provider_id))
        .map(|provider| provider.ordered.clone())
        .unwrap_or_default()
}

pub fn inherits_upstream(provider_id: &str) -> bool {
    registry()
        .and_then(|registry| registry.providers.get(provider_id))
        .is_some_and(|provider| provider.inherits_upstream)
}

fn registry() -> Option<&'static ProviderModelRegistry> {
    REGISTRY
        .get_or_init(|| match parse_sources(SOURCES) {
            Ok(registry) => Some(registry),
            Err(code) => {
                ::log::error!("[provider-models] chargement refusé code={code}");
                None
            }
        })
        .as_ref()
}

fn parse_sources(
    sources: &[EmbeddedProviderModels],
) -> Result<ProviderModelRegistry, &'static str> {
    if sources.is_empty() || sources.len() > MAX_PROVIDERS {
        return Err("provider_count");
    }
    let mut providers = HashMap::with_capacity(sources.len());
    for source in sources {
        let file: ProviderModelFile =
            serde_json::from_str(source.json).map_err(|_| "invalid_json")?;
        validate_file(source.provider_id, &file)?;
        if providers.contains_key(source.provider_id) {
            return Err("duplicate_provider");
        }
        let total_ids = file
            .models
            .iter()
            .map(|model| 1 + model.aliases.len())
            .sum();
        let mut by_id = HashMap::with_capacity(total_ids);
        for model in &file.models {
            if by_id.insert(model.id.clone(), model.clone()).is_some() {
                return Err("duplicate_model");
            }
            for alias in &model.aliases {
                if by_id.insert(alias.clone(), model.clone()).is_some() {
                    return Err("duplicate_model");
                }
            }
        }
        providers.insert(
            source.provider_id.to_string(),
            ProviderModels {
                ordered: file.models,
                by_id,
                inherits_upstream: file.inherits_upstream,
            },
        );
    }
    Ok(ProviderModelRegistry { providers })
}

fn validate_file(expected_provider: &str, file: &ProviderModelFile) -> Result<(), &'static str> {
    if file.schema_version != 1 {
        return Err("schema_version");
    }
    if file.provider != expected_provider || !valid_provider_id(&file.provider) {
        return Err("provider_id");
    }
    if file.provider != crate::services::llm::providers::openai::PROVIDER_ID
        && file.models.iter().any(|model| model.supports_fast_mode)
    {
        return Err("fast_mode_provider");
    }
    if !valid_date(&file.verified_at)
        || file.source_urls.is_empty()
        || file.source_urls.len() > MAX_SOURCE_URLS
        || !file.source_urls.iter().all(|url| valid_source_url(url))
    {
        return Err("provenance");
    }
    if file.models.len() > MAX_MODELS_PER_PROVIDER
        || (file.models.is_empty() && !file.inherits_upstream)
    {
        return Err("model_count");
    }
    let total_ids = file
        .models
        .iter()
        .map(|model| 1 + model.aliases.len())
        .sum::<usize>();
    if total_ids > MAX_MODELS_PER_PROVIDER {
        return Err("model_count");
    }
    let mut ids = HashSet::with_capacity(total_ids);
    for model in &file.models {
        if !super::runtime_models::valid_model_id(&model.id) || !ids.insert(model.id.as_str()) {
            return Err("model_id");
        }
        if model.aliases.len() > MAX_ALIASES_PER_MODEL {
            return Err("model_id");
        }
        for alias in &model.aliases {
            if !super::runtime_models::valid_model_id(alias) || !ids.insert(alias.as_str()) {
                return Err("model_id");
            }
        }
        if model.context_window == 0 || model.context_window > MAX_CONTEXT_TOKENS {
            return Err("context_window");
        }
        if model
            .max_output_tokens
            .is_some_and(|limit| limit == 0 || limit > model.context_window)
        {
            return Err("output_limit");
        }
        if model.default_output_tokens.is_some_and(|default| {
            default == 0
                || default > model.context_window
                || model
                    .max_output_tokens
                    .is_some_and(|maximum| default > maximum)
        }) {
            return Err("output_default");
        }
        valid_reasoning_contract(
            model.supports_thinking,
            &model.reasoning_modes,
            model.default_reasoning_mode.as_deref(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "provider_model_registry_inventory_tests.rs"]
mod inventory_tests;
#[cfg(test)]
#[path = "provider_model_registry_tests.rs"]
mod tests;
