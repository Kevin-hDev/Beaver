use std::sync::LazyLock;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::model_catalog_wire::{ModelsResponse, WireModel};
use super::request_http;
use crate::services::llm::types::ModelInfo;
use crate::services::secure_http::{read_json_bounded, CODEX_MODELS_BODY_LIMIT};

const CACHE_TTL: Duration = Duration::from_secs(300);
const FAILURE_TTL: Duration = Duration::from_secs(30);
const DEFAULT_EFFECTIVE_PERCENT: u64 = 95;
const MAX_CONTEXT_WINDOW: u64 = 10_000_000;
const ALLOWED_MODES: &[&str] = &[
    "none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra",
];

#[derive(Clone)]
struct CatalogModel {
    info: ModelInfo,
    visible: bool,
}

struct CachedCatalog {
    models: Vec<CatalogModel>,
    fetched_at: Instant,
}

#[derive(Default)]
struct CacheState {
    catalog: Option<CachedCatalog>,
    failed_at: Option<Instant>,
}

static CACHE: LazyLock<Mutex<CacheState>> = LazyLock::new(|| Mutex::new(CacheState::default()));

pub async fn available_models() -> Result<Vec<ModelInfo>, String> {
    let models = load_catalog().await?;
    let visible = models
        .into_iter()
        .filter(|model| model.visible)
        .map(|model| model.info)
        .collect::<Vec<_>>();
    if visible.is_empty() {
        Err(unavailable())
    } else {
        Ok(visible)
    }
}

pub async fn context_length(model_id: &str) -> u64 {
    if let Ok(models) = load_catalog().await {
        if let Some(context) = find_context(&models, model_id) {
            return context;
        }
    }
    fallback_models()
        .into_iter()
        .find(|model| model.id == model_id)
        .and_then(|model| model.context_length)
        .map(u64::from)
        .unwrap_or(128_000)
}

pub async fn supports_fast_mode(model_id: &str) -> Result<bool, String> {
    if !crate::services::llm::runtime_models::valid_model_id(model_id) {
        return Ok(false);
    }
    Ok(load_catalog()
        .await?
        .iter()
        .find(|model| model.info.id == model_id)
        .is_some_and(|model| model.info.supports_fast_mode))
}

pub fn fallback_models() -> Vec<ModelInfo> {
    super::model_catalog_fallback::models()
}

pub async fn invalidate() {
    *CACHE.lock().await = CacheState::default();
}

async fn load_catalog() -> Result<Vec<CatalogModel>, String> {
    let mut cache = CACHE.lock().await;
    if let Some(cached) = cache.catalog.as_ref() {
        if cached.fetched_at.elapsed() <= CACHE_TTL {
            return Ok(cached.models.clone());
        }
    }
    if cache
        .failed_at
        .is_some_and(|failed_at| failed_at.elapsed() <= FAILURE_TTL)
    {
        return cache
            .catalog
            .as_ref()
            .map(|cached| cached.models.clone())
            .ok_or_else(unavailable);
    }
    match fetch_catalog().await {
        Ok(models) => {
            cache.catalog = Some(CachedCatalog {
                models: models.clone(),
                fetched_at: Instant::now(),
            });
            cache.failed_at = None;
            Ok(models)
        }
        Err(error) => {
            cache.failed_at = Some(Instant::now());
            cache
                .catalog
                .as_ref()
                .map(|cached| cached.models.clone())
                .ok_or(error)
        }
    }
}

async fn fetch_catalog() -> Result<Vec<CatalogModel>, String> {
    let response = request_http::get_models().await?;
    let wire: ModelsResponse = read_json_bounded(response, CODEX_MODELS_BODY_LIMIT)
        .await
        .map_err(|_| unavailable())?;
    parse_response(wire)
}

fn parse_response(response: ModelsResponse) -> Result<Vec<CatalogModel>, String> {
    let mut models = Vec::with_capacity(response.models.0.len());
    for wire in response.models.0 {
        let Some(model) = convert_model(wire) else {
            continue;
        };
        if !models
            .iter()
            .any(|existing: &CatalogModel| existing.info.id == model.info.id)
        {
            models.push(model);
        }
    }
    if models.is_empty() {
        Err(unavailable())
    } else {
        Ok(models)
    }
}

fn convert_model(wire: WireModel) -> Option<CatalogModel> {
    if !crate::services::llm::runtime_models::valid_model_id(&wire.slug) {
        return None;
    }
    let raw_context = wire.context_window.or(wire.max_context_window)?;
    let percent = wire
        .effective_context_window_percent
        .unwrap_or(DEFAULT_EFFECTIVE_PERCENT);
    let context_length = effective_context(raw_context, percent)?;
    let supports_fast_mode = super::model_catalog_fast::supports_fast_mode(&wire);
    let modes = validated_modes(wire.supported_reasoning_levels.0);
    let display_name = if valid_display_name(&wire.display_name) {
        wire.display_name.clone()
    } else {
        wire.slug.clone()
    };
    let supports_vision = wire.input_modalities.0.iter().any(|mode| mode == "image");
    let supports_tools = super::supports_tools(&wire.slug);
    Some(CatalogModel {
        visible: wire.visibility.as_deref().unwrap_or("list") == "list",
        info: ModelInfo {
            id: wire.slug,
            display_name: Some(display_name),
            owned_by: Some("openai".to_string()),
            context_length: Some(context_length),
            max_output_tokens: None,
            supports_tools,
            supports_vision,
            supports_thinking: !modes.is_empty(),
            supports_fast_mode,
            reasoning_modes: modes,
            default_reasoning_mode: None,
            context_usage_includes_reasoning: true,
            is_free: false,
        },
    })
}

fn effective_context(raw: u64, percent: u64) -> Option<u32> {
    if !(1_024..=MAX_CONTEXT_WINDOW).contains(&raw) || !(1..=100).contains(&percent) {
        return None;
    }
    u32::try_from(raw.checked_mul(percent)?.checked_div(100)?).ok()
}

fn validated_modes(levels: Vec<super::model_catalog_wire::ReasoningLevel>) -> Vec<String> {
    let mut modes = Vec::with_capacity(levels.len());
    for level in levels {
        if ALLOWED_MODES.contains(&level.effort.as_str()) && !modes.contains(&level.effort) {
            modes.push(level.effort);
        }
    }
    modes
}

fn valid_display_name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn find_context(models: &[CatalogModel], model_id: &str) -> Option<u64> {
    models
        .iter()
        .find(|model| model.info.id == model_id)
        .and_then(|model| model.info.context_length)
        .map(u64::from)
}

fn unavailable() -> String {
    "model_catalog_unavailable".to_string()
}

#[cfg(test)]
#[path = "model_catalog_tests.rs"]
mod tests;
