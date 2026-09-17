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

#[cfg(test)]
pub(super) async fn seed_for_test(model: XaiCatalogModel) {
    *CACHE.lock().await = Some(CachedCatalog {
        fetched_at: Instant::now(),
        models: vec![model],
    });
}

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
    let local = crate::services::llm::provider_model_lookup::local_reasoning("xai", model_id);
    model.reasoning_contract = merge_reasoning_contract(model.reasoning_contract, local);
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
            #[cfg(debug_assertions)]
            route::RouteError::FixtureBudget(message) => LlmError::Provider(message),
        })?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 | 403 => LlmError::Unauthorized,
            429 => LlmError::RateLimit {
                retry_after_secs: catalog_retry_after(response.headers()),
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

pub(super) fn catalog_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    crate::services::llm::provider_error::retry_after_seconds(headers)
}

fn to_model_info(model: &XaiCatalogModel) -> ModelInfo {
    let local = crate::services::llm::provider_model_lookup::local_capabilities("xai", &model.id)
        .unwrap_or_default();
    let local_reasoning =
        crate::services::llm::provider_model_lookup::local_reasoning("xai", &model.id);
    let reasoning_contract =
        merge_reasoning_contract(model.reasoning_contract.clone(), local_reasoning);
    let supports_thinking = local.supports_thinking || reasoning_contract.is_some();
    ModelInfo {
        id: model.id.clone(),
        display_name: Some(model.display_name.clone()),
        owned_by: None,
        context_length: Some(model.context_window),
        max_output_tokens: model.max_output_tokens,
        supported_parameters: None,
        catalog_capabilities: Default::default(),
        supports_tools: local.supports_tools,
        supports_vision: local.supports_vision,
        supports_thinking,
        reasoning_contract: reasoning_contract.or_else(|| {
            crate::services::llm::model_reasoning_contract::ModelReasoningContract::from_modes(
                supports_thinking,
                &[],
                None,
            )
        }),
        supports_fast_mode: false,
        context_usage_includes_reasoning: true,
        is_free: false,
    }
}

pub(super) fn merge_reasoning_contract(
    mut remote: Option<crate::services::llm::model_reasoning_contract::ModelReasoningContract>,
    local: Option<crate::services::llm::model_reasoning_contract::ModelReasoningContract>,
) -> Option<crate::services::llm::model_reasoning_contract::ModelReasoningContract> {
    let Some(contract) = remote.as_mut() else {
        return local;
    };
    if contract.default_effort.is_none() {
        let fallback = local.and_then(|value| value.default_effort);
        contract.default_effort = fallback.filter(|mode| contract.supports_mode(mode.as_name()));
        contract.default_enabled = contract
            .default_enabled
            .or(contract.default_effort.map(|mode| mode.as_name() != "off"));
    }
    remote
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
