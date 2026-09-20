use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use zeroize::Zeroizing;

use super::types::OAuthTokens;
use crate::services::api_keys;
use crate::services::secure_http_destination::FixedDestination;

pub(crate) type DestinationFuture =
    Pin<Box<dyn Future<Output = Result<FixedDestination, String>> + Send>>;

static REFRESH_LOCKS: std::sync::LazyLock<
    std::sync::Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

fn get_refresh_lock(connector_id: &str) -> Result<Arc<tokio::sync::Mutex<()>>, String> {
    crate::services::mcp_bridge::config::validate_connector_id(connector_id)?;
    let mut map = REFRESH_LOCKS
        .lock()
        .map_err(|_| "renouvellement MCP indisponible")?;
    if !map.contains_key(connector_id) && map.len() >= 32 {
        let idle = map
            .iter()
            .find(|(_, lock)| Arc::strong_count(lock) == 1)
            .map(|(id, _)| id.clone())
            .ok_or("trop de renouvellements MCP")?;
        map.remove(&idle);
    }
    Ok(Arc::clone(
        map.entry(connector_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
    ))
}

fn write_tokens(connector_id: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let json = tokens.to_json()?;
    api_keys::set_mcp_token(connector_id, json.as_str())
}

pub(super) fn store_connection_tokens(
    _mutation: &crate::services::mcp_bridge::registry::IdentityMutation,
    connector_id: &str,
    tokens: &OAuthTokens,
) -> Result<(), String> {
    if tokens.issuer.is_none() {
        return Err(super::types::REAUTHENTICATION_REQUIRED.to_string());
    }
    write_tokens(connector_id, tokens)
}

pub(super) fn store_refreshed_tokens_if_generation(
    connector_id: &str,
    tokens: &OAuthTokens,
    expected_generation: u64,
) -> Result<(), String> {
    save_if_generation(expected_generation, || write_tokens(connector_id, tokens))
}

pub(super) fn save_if_generation<T>(
    generation: u64,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    crate::services::mcp_bridge::registry::with_current_generation(generation, action)
}

pub fn get_tokens(connector_id: &str) -> Result<OAuthTokens, String> {
    let json = api_keys::get_mcp_token(connector_id)?;
    OAuthTokens::from_json(json.as_str())
}

pub fn delete_tokens(connector_id: &str) -> Result<(), String> {
    api_keys::delete_mcp_token(connector_id)
}

pub fn has_tokens(connector_id: &str) -> bool {
    api_keys::has_mcp_token(connector_id)
}

pub async fn get_valid_token(connector_id: &str) -> Result<Zeroizing<String>, String> {
    let dependencies = RefreshDependencies {
        read: &get_tokens,
        read_current: &|id, generation| {
            crate::services::mcp_bridge::registry::with_current_generation(generation, || {
                get_tokens(id)
            })
        },
        save: &store_refreshed_tokens_if_generation,
        generation: &crate::services::mcp_bridge::registry::current_generation,
        validate: &super::trusted_oauth::validate_endpoint,
        validate_issuer: &super::trusted_oauth::validate_issuer,
        client: &|id, url| {
            let id = id.to_string();
            let url = url.to_string();
            Box::pin(async move {
                super::network_guard::destination(
                    &id,
                    "",
                    super::network_guard::DestinationRole::OAuth,
                    &url,
                )
                .await
            })
        },
    };
    get_valid_token_with(connector_id, &dependencies).await
}

pub(crate) struct RefreshDependencies<'a> {
    pub read: &'a (dyn Fn(&str) -> Result<OAuthTokens, String> + Send + Sync),
    pub read_current: &'a (dyn Fn(&str, u64) -> Result<OAuthTokens, String> + Send + Sync),
    pub save: &'a (dyn Fn(&str, &OAuthTokens, u64) -> Result<(), String> + Send + Sync),
    pub generation: &'a (dyn Fn() -> Result<u64, String> + Send + Sync),
    pub validate: &'a (dyn Fn(&str, &str) -> Result<(), String> + Send + Sync),
    pub validate_issuer: &'a (dyn Fn(&str, &str) -> Result<(), String> + Send + Sync),
    pub client: &'a (dyn Fn(&str, &str) -> DestinationFuture + Send + Sync),
}

pub(crate) async fn get_valid_token_with(
    connector_id: &str,
    dependencies: &RefreshDependencies<'_>,
) -> Result<Zeroizing<String>, String> {
    let tokens = (dependencies.read)(connector_id)?;
    let issuer = tokens
        .issuer
        .as_deref()
        .ok_or(super::types::REAUTHENTICATION_REQUIRED)?;
    (dependencies.validate_issuer)(connector_id, issuer)
        .map_err(|_| super::types::REAUTHENTICATION_REQUIRED.to_string())?;
    (dependencies.validate)(connector_id, &tokens.token_endpoint)
        .map_err(|_| super::types::REAUTHENTICATION_REQUIRED.to_string())?;
    let result = tokens.access_token.clone();
    if let Some(exp) = tokens.expires_at {
        let now = chrono::Utc::now().timestamp();
        if now < exp.saturating_sub(30) {
            return Ok(result);
        }
        let lock = get_refresh_lock(connector_id)?;
        let _guard = lock.lock().await;
        let generation = (dependencies.generation)()?;
        let fresh = (dependencies.read_current)(connector_id, generation)?;
        if fresh.issuer.as_deref() != Some(issuer) {
            return Err(super::types::REAUTHENTICATION_REQUIRED.to_string());
        }
        if let Some(fexp) = fresh.expires_at {
            if chrono::Utc::now().timestamp() < fexp.saturating_sub(30) {
                return Ok(fresh.access_token.clone());
            }
        }
        let refresh = fresh
            .refresh_token
            .as_ref()
            .ok_or(super::types::REAUTHENTICATION_REQUIRED)?;
        return super::storage_refresh::refresh_access_token(
            connector_id,
            &fresh,
            refresh.as_str(),
            generation,
            dependencies,
        )
        .await;
    }
    Ok(result)
}
