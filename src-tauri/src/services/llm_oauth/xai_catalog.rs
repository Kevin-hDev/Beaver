use std::sync::LazyLock;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::xai_catalog_wire::{parse_catalog, XaiCatalogModel};
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::route;
use crate::services::llm::types::{LlmError, ModelInfo};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient};

const CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const STALE_TTL: Duration = Duration::from_secs(15 * 60);
const CATALOG_TIMEOUT: Duration = Duration::from_secs(15);
const CATALOG_BODY_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone)]
struct CachedCatalog {
    fetched_at: Instant,
    models: Vec<XaiCatalogModel>,
}

static CACHE: LazyLock<Mutex<Option<CachedCatalog>>> = LazyLock::new(|| Mutex::new(None));

pub async fn list_models() -> Result<Vec<ModelInfo>, LlmError> {
    Ok(catalog().await?.iter().map(to_model_info).collect())
}

pub async fn model(model_id: &str) -> Result<XaiCatalogModel, LlmError> {
    if !crate::services::llm::runtime_models::valid_model_id(model_id) {
        return Err(configuration_error());
    }
    let mut model = catalog()
        .await?
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or_else(configuration_error)?;
    if let Some(local) =
        crate::services::llm::provider_model_lookup::local_reasoning("xai", model_id)
    {
        if model.reasoning_modes.is_empty() {
            model.reasoning_modes = local.modes;
        }
        if model.default_reasoning_mode.is_none() {
            model.default_reasoning_mode = local.default_mode;
        }
    }
    Ok(model)
}

async fn catalog() -> Result<Vec<XaiCatalogModel>, LlmError> {
    if let Some(models) = cached_within(CACHE_TTL).await {
        return Ok(models);
    }
    match fetch().await {
        Ok(models) => {
            *CACHE.lock().await = Some(CachedCatalog {
                fetched_at: Instant::now(),
                models: models.clone(),
            });
            Ok(models)
        }
        Err(error) => cached_within(STALE_TTL).await.ok_or(error),
    }
}

async fn cached_within(max_age: Duration) -> Option<Vec<XaiCatalogModel>> {
    CACHE
        .lock()
        .await
        .as_ref()
        .filter(|cached| cached.fetched_at.elapsed() <= max_age)
        .map(|cached| cached.models.clone())
}

async fn fetch() -> Result<Vec<XaiCatalogModel>, LlmError> {
    let route = route::resolve("xai-oauth").ok_or_else(configuration_error)?;
    let client = AuthenticatedClient::new(CATALOG_TIMEOUT).map_err(|_| network_error())?;
    let url = format!("{}{}", route.base_url, route.models_endpoint);
    let response = route
        .send_authenticated(
            &client,
            RequestPurpose::AccountMetadata,
            |token, headers| client.get(&url).headers(headers).bearer_auth(token),
        )
        .await
        .map_err(|error| match error {
            route::RouteError::Unauthorized => LlmError::Unauthorized,
            route::RouteError::Forbidden => LlmError::KnownProvider(
                crate::services::llm::provider_error::ProviderErrorCode::ProviderAccessUnavailable,
            ),
            route::RouteError::Network => network_error(),
        })?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 | 403 => LlmError::Unauthorized,
            429 => LlmError::RateLimit {
                retry_after_secs: response
                    .headers()
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse().ok()),
            },
            _ => LlmError::KnownProvider(
                crate::services::llm::provider_error::ProviderErrorCode::ModelCatalogUnavailable,
            ),
        });
    }
    let body = read_json_bounded(response, CATALOG_BODY_LIMIT)
        .await
        .map_err(|_| catalog_error())?;
    parse_catalog(&body).map_err(|_| catalog_error())
}

fn to_model_info(model: &XaiCatalogModel) -> ModelInfo {
    let local = crate::services::llm::provider_model_lookup::local_capabilities("xai", &model.id)
        .unwrap_or_else(
            || crate::services::llm::provider_model_lookup::ModelCapabilities {
                supports_tools: crate::services::llm::providers::xai::supports_tools(&model.id),
                supports_vision: crate::services::llm::providers::xai::supports_vision(&model.id),
                supports_thinking: crate::services::llm::providers::xai::supports_thinking(
                    &model.id,
                ),
            },
        );
    let local_reasoning =
        crate::services::llm::provider_model_lookup::local_reasoning("xai", &model.id);
    ModelInfo {
        id: model.id.clone(),
        display_name: Some(model.display_name.clone()),
        owned_by: None,
        context_length: Some(model.context_window),
        max_output_tokens: model.max_output_tokens,
        supports_tools: local.supports_tools,
        supports_vision: local.supports_vision,
        supports_thinking: local.supports_thinking || !model.reasoning_modes.is_empty(),
        supports_fast_mode: false,
        reasoning_modes: if model.reasoning_modes.is_empty() {
            local_reasoning
                .as_ref()
                .map(|reasoning| reasoning.modes.clone())
                .unwrap_or_default()
        } else {
            model.reasoning_modes.clone()
        },
        default_reasoning_mode: model
            .default_reasoning_mode
            .clone()
            .or_else(|| local_reasoning.and_then(|reasoning| reasoning.default_mode)),
        is_free: false,
    }
}

fn configuration_error() -> LlmError {
    LlmError::KnownProvider(
        crate::services::llm::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
    )
}

fn catalog_error() -> LlmError {
    LlmError::KnownProvider(
        crate::services::llm::provider_error::ProviderErrorCode::ModelCatalogUnavailable,
    )
}

fn network_error() -> LlmError {
    LlmError::KnownProvider(
        crate::services::llm::provider_error::ProviderErrorCode::ProviderConnectionFailed,
    )
}
