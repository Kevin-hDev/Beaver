use std::collections::HashMap;
use std::sync::Arc;

use zeroize::Zeroizing;

use super::types::{OAuthTokens, TokenResponse};
use crate::services::api_keys;
use crate::services::secure_http::AuthenticatedClient;

static REFRESH_LOCKS: std::sync::LazyLock<
    std::sync::Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

fn get_refresh_lock(connector_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    let mut map = REFRESH_LOCKS.lock().unwrap_or_else(|e| e.into_inner());
    if map.len() > 64 {
        map.clear();
    }
    Arc::clone(
        map.entry(connector_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
    )
}

pub fn store_tokens(connector_id: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let json = tokens.to_json()?;
    api_keys::set_mcp_token(connector_id, json.as_str())
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
    let tokens = get_tokens(connector_id)?;
    let result = tokens.access_token.clone();
    if let Some(exp) = tokens.expires_at {
        let now = chrono::Utc::now().timestamp();
        if now < exp - 30 {
            return Ok(result);
        }
        let lock = get_refresh_lock(connector_id);
        let _guard = lock.lock().await;
        let fresh = get_tokens(connector_id)?;
        if let Some(fexp) = fresh.expires_at {
            if chrono::Utc::now().timestamp() < fexp - 30 {
                return Ok(fresh.access_token.clone());
            }
        }
        let refresh = fresh
            .refresh_token
            .as_ref()
            .ok_or("token expiré et pas de refresh_token")?;
        return refresh_access_token(connector_id, &fresh, refresh.as_str()).await;
    }
    Ok(result)
}

async fn refresh_access_token(
    connector_id: &str,
    old: &OAuthTokens,
    refresh_token: &str,
) -> Result<Zeroizing<String>, String> {
    super::trusted_oauth::validate_endpoint(connector_id, &old.token_endpoint)?;

    let client = AuthenticatedClient::new(std::time::Duration::from_secs(15))
        .map_err(|_| "erreur interne".to_string())?;

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
    store_tokens(connector_id, &new_tokens)?;
    Ok(result)
}
