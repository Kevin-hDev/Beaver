use std::collections::HashMap;
use std::sync::Arc;

use zeroize::Zeroizing;

use super::types::{OAuthTokens, TokenResponse};
use crate::services::api_keys;
use crate::services::secure_http::AuthenticatedClient;

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
        client: &|| {
            AuthenticatedClient::new(std::time::Duration::from_secs(15))
                .map_err(|_| "erreur interne".to_string())
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
    pub client: &'a (dyn Fn() -> Result<AuthenticatedClient, String> + Send + Sync),
}

pub(crate) async fn get_valid_token_with(
    connector_id: &str,
    dependencies: &RefreshDependencies<'_>,
) -> Result<Zeroizing<String>, String> {
    let tokens = (dependencies.read)(connector_id)?;
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
        if let Some(fexp) = fresh.expires_at {
            if chrono::Utc::now().timestamp() < fexp.saturating_sub(30) {
                return Ok(fresh.access_token.clone());
            }
        }
        let refresh = fresh
            .refresh_token
            .as_ref()
            .ok_or("token expiré et pas de refresh_token")?;
        return refresh_access_token(
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

async fn refresh_access_token(
    connector_id: &str,
    old: &OAuthTokens,
    refresh_token: &str,
    generation: u64,
    dependencies: &RefreshDependencies<'_>,
) -> Result<Zeroizing<String>, String> {
    (dependencies.validate)(connector_id, &old.token_endpoint)?;
    let client = (dependencies.client)()?;

    let mut params: Vec<(&str, &str)> = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", old.client_id.as_str()),
    ];
    let secret_ref = old
        .client_secret
        .as_ref()
        .map(|s| Zeroizing::new(s.as_str().to_string()));
    if let Some(ref secret) = secret_ref {
        params.push(("client_secret", secret.as_str()));
    }

    let request = client
        .post(&old.token_endpoint)
        .header("Accept", "application/json")
        .form(&params);
    let resp = client
        .send_success(request)
        .await
        .map_err(|_| "échec du rafraîchissement du token".to_string())?;

    let mut raw: TokenResponse = super::bounded_json(resp).await?;

    if raw.access_token.is_empty() {
        return Err("token manquant dans la réponse".to_string());
    }

    if raw.refresh_token.is_none() {
        raw.refresh_token = Some(refresh_token.to_string());
    }

    let cs = old.client_secret.as_ref().map(|s| s.as_str());
    let new_tokens = OAuthTokens::from_response(&mut raw, &old.token_endpoint, &old.client_id, cs);
    let result = new_tokens.access_token.clone();
    (dependencies.save)(connector_id, &new_tokens, generation)?;
    Ok(result)
}
