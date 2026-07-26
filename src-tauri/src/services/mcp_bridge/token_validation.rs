use std::time::Duration;

use serde_json::Value;
use zeroize::Zeroizing;

use super::config::StoredConnector;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, TOKEN_BODY_LIMIT};

const TIMEOUT: Duration = Duration::from_secs(10);
const TOKEN_ERROR: &str = "token MCP invalide";

pub async fn validate_connector_tokens(connector: &StoredConnector) -> Result<(), String> {
    match connector.id.as_str() {
        "huggingface" => validate_huggingface(&env_token(connector, "HF_TOKEN")?).await,
        "producthunt" => validate_producthunt(&env_token(connector, "PRODUCT_HUNT_TOKEN")?).await,
        _ => Ok(()),
    }
}

fn env_token(connector: &StoredConnector, env_key: &str) -> Result<Zeroizing<String>, String> {
    let allowed = connector
        .env_keys
        .as_deref()
        .unwrap_or_default()
        .iter()
        .any(|key| key == env_key);
    if !allowed {
        return Err(TOKEN_ERROR.to_string());
    }
    let vault_key = format!("mcp_{}_{}", connector.id, env_key.to_lowercase());
    let token = crate::services::api_keys::get_key(&vault_key).map_err(|_| TOKEN_ERROR)?;
    if token.trim().is_empty() {
        return Err(TOKEN_ERROR.to_string());
    }
    Ok(token)
}

async fn validate_huggingface(token: &str) -> Result<(), String> {
    let client = client()?;
    let request = client
        .get("https://huggingface.co/api/whoami-v2")
        .bearer_auth(token);
    let resp = client
        .send(request)
        .await
        .map_err(|_| TOKEN_ERROR.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(TOKEN_ERROR.to_string())
    }
}

async fn validate_producthunt(token: &str) -> Result<(), String> {
    let client = client()?;
    let request = client
        .post("https://api.producthunt.com/v2/api/graphql")
        .bearer_auth(token)
        .json(&serde_json::json!({
            "query": "query TokenProbe { posts(first: 1) { edges { node { id } } } }"
        }));
    let resp = client
        .send(request)
        .await
        .map_err(|_| TOKEN_ERROR.to_string())?;
    if !resp.status().is_success() {
        return Err(TOKEN_ERROR.to_string());
    }
    let body: Value = read_json_bounded(resp, TOKEN_BODY_LIMIT)
        .await
        .map_err(|_| TOKEN_ERROR.to_string())?;
    if body.get("errors").is_some() {
        return Err(TOKEN_ERROR.to_string());
    }
    let has_posts = body
        .get("data")
        .and_then(|data| data.get("posts"))
        .and_then(|posts| posts.get("edges"))
        .and_then(|edges| edges.as_array())
        .is_some();
    if has_posts {
        Ok(())
    } else {
        Err(TOKEN_ERROR.to_string())
    }
}

fn client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new(TIMEOUT).map_err(|_| "erreur interne".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_connectors_without_known_token_probe() {
        let connector = StoredConnector {
            id: "context7".to_string(),
            status: "connected".to_string(),
            enabled_in_chat: true,
            endpoint: None,
            install_command: Some("npx @upstash/context7-mcp@2.2.5".to_string()),
            env_keys: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(validate_connector_tokens(&connector)).is_ok());
    }
}
