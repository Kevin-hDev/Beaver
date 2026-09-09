use super::provider_error::ProviderErrorCode;
use super::types::{LlmError, ModelInfo};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, LLM_BODY_LIMIT};
use futures_util::future::{BoxFuture, FutureExt, Shared};
use std::sync::Arc;

type PendingRefresh = Arc<Shared<BoxFuture<'static, Result<Vec<ModelInfo>, LlmError>>>>;
// One in-flight download, not another catalog cache. Concurrent callers share
// both success and failure; completed results are retained only in runtime_models.
static INITIALIZATION: tokio::sync::Mutex<Option<PendingRefresh>> =
    tokio::sync::Mutex::const_new(None);

pub async fn list_models() -> Result<Vec<ModelInfo>, LlmError> {
    let client = AuthenticatedClient::new(super::timeouts::request_timeout_for("openrouter"))
        .map_err(|_| unavailable())?;
    list_models_shared(&client, &public_models_url()?).await
}

pub async fn ensure_model(model_id: &str) -> Result<ModelInfo, LlmError> {
    let client = AuthenticatedClient::new(super::timeouts::request_timeout_for("openrouter"))
        .map_err(|_| unavailable())?;
    ensure_model_from(&client, &public_models_url()?, model_id).await
}

pub async fn ensure_model_for_route(provider_id: &str, model_id: &str) -> Result<bool, LlmError> {
    if !super::openrouter_model_metadata::owns_catalog_metadata(provider_id) {
        return Ok(false);
    }
    ensure_model(model_id).await.map(|_| true)
}

fn public_models_url() -> Result<String, LlmError> {
    let route = super::route::resolve("openrouter").ok_or_else(unavailable)?;
    Ok(format!("{}{}", route.base_url, route.models_endpoint))
}

async fn ensure_model_from(
    client: &AuthenticatedClient,
    url: &str,
    model_id: &str,
) -> Result<ModelInfo, LlmError> {
    if let Some(model) = super::runtime_models::lookup("openrouter", model_id) {
        return Ok(model);
    }
    list_models_shared(client, url).await?;
    super::runtime_models::lookup("openrouter", model_id).ok_or_else(unavailable)
}

async fn list_models_shared(
    client: &AuthenticatedClient,
    url: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let pending = {
        let mut slot = INITIALIZATION.lock().await;
        Arc::clone(slot.get_or_insert_with(|| {
            let client = client.clone();
            let url = url.to_owned();
            Arc::new(
                async move { refresh_models_from(&client, &url).await }
                    .boxed()
                    .shared(),
            )
        }))
    };
    let result = pending.as_ref().clone().await;
    let mut slot = INITIALIZATION.lock().await;
    if slot
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, &pending))
    {
        *slot = None;
    }
    result
}

async fn refresh_models_from(
    client: &AuthenticatedClient,
    url: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let response = client
        .send(client.get(url))
        .await
        .map_err(|_| unavailable())?;
    if !response.status().is_success() {
        // This GET carries no key; only the authenticated /key probe can judge it.
        return Err(unavailable());
    }
    let body: serde_json::Value = read_json_bounded(response, LLM_BODY_LIMIT)
        .await
        .map_err(|_| unavailable())?;
    let parsed = super::openai_compat_parsing::parse_models_list(&body, "openrouter")
        .map_err(|_| unavailable())?;
    super::model_catalog::enrich_models("openrouter", parsed, false)
        .await
        .map_err(|_| unavailable())
}

fn unavailable() -> LlmError {
    LlmError::KnownProvider(ProviderErrorCode::ModelCatalogUnavailable)
}

#[cfg(test)]
pub(super) async fn list_models_from_url_for_test(url: &str) -> Result<Vec<ModelInfo>, LlmError> {
    let client = AuthenticatedClient::new_loopback(std::time::Duration::from_secs(1))
        .map_err(|_| unavailable())?;
    list_models_shared(&client, url).await
}

#[cfg(test)]
pub(super) async fn ensure_model_from_url_for_test(
    model_id: &str,
    url: &str,
) -> Result<ModelInfo, LlmError> {
    let client = AuthenticatedClient::new_loopback(std::time::Duration::from_secs(1))
        .map_err(|_| unavailable())?;
    ensure_model_from(&client, url, model_id).await
}
