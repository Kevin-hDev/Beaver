use super::types::{MAX_PROJECT_RESULTS, MAX_SESSION_RESULTS, MCP_TOOL_TIMEOUT_MS};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;
use zeroize::Zeroizing;

pub enum CoreResponse {
    Json(Value),
    Secret(Zeroizing<String>),
}

pub async fn call(
    _identity: &super::host_identity::HostIdentity,
    api_level: &super::types::ExtensionApiLevel,
    method: &str,
    params: Option<&Value>,
) -> Result<CoreResponse, ()> {
    let params = params.unwrap_or(&Value::Null);
    if params.get("extensionId").is_some()
        || (method.starts_with("unstable.")
            && *api_level != super::types::ExtensionApiLevel::Advanced)
    {
        return Err(());
    }
    let normalized = method.strip_prefix("unstable.").unwrap_or(method);
    match normalized {
        "app.info" => Ok(CoreResponse::Json(json!({
            "apiVersion": super::types::BEAVER_API_VERSION,
            "dataDir": crate::services::paths::data_dir().to_string_lossy(),
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
        }))),
        "sessions.list" => {
            let sessions = crate::services::agent_local::session_store::list()
                .await
                .map_err(|_| ())?
                .into_iter()
                .take(MAX_SESSION_RESULTS)
                .collect::<Vec<_>>();
            json_response(sessions)
        }
        "sessions.get" => {
            let id = string_param(params, "sessionId")?;
            let session = crate::services::agent_local::session_store::get(id)
                .await
                .map_err(|_| ())?;
            json_response(session)
        }
        "projects.list" => {
            let projects = crate::services::agent_local::project_store::list()
                .await
                .map_err(|_| ())?
                .into_iter()
                .take(MAX_PROJECT_RESULTS)
                .collect::<Vec<_>>();
            json_response(projects)
        }
        "mcp.connectors.list" => {
            let connectors = crate::services::mcp_bridge::config::load().map_err(|_| ())?;
            json_response(connectors)
        }
        "mcp.tool.call" => call_mcp_tool(params).await,
        "channels.config.get" => {
            let config = crate::services::config::read_config().map_err(|_| ())?;
            json_response(config.gateway)
        }
        "secrets.provider.get" => provider_secret(params),
        "secrets.mcp.oauth.get" => mcp_oauth_secret(params).await,
        "secrets.mcp.env.get" => mcp_env_secret(params),
        "secrets.channel.get" => channel_secret(params),
        _ => Err(()),
    }
}

fn provider_secret(params: &Value) -> Result<CoreResponse, ()> {
    let provider = string_param(params, "providerId")?;
    crate::services::api_keys::validate::validate_provider(provider).map_err(|_| ())?;
    let secret = crate::services::api_keys::get_key(provider).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

async fn mcp_oauth_secret(params: &Value) -> Result<CoreResponse, ()> {
    let connector = string_param(params, "connectorId")?;
    crate::services::mcp_bridge::config::validate_connector_id(connector).map_err(|_| ())?;
    let secret = crate::services::mcp_oauth::storage::get_valid_token(connector)
        .await
        .map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

fn mcp_env_secret(params: &Value) -> Result<CoreResponse, ()> {
    let connector_id = string_param(params, "connectorId")?;
    let env_key = string_param(params, "envKey")?;
    let connector = crate::services::mcp_bridge::config::find(connector_id)
        .map_err(|_| ())?
        .ok_or(())?;
    let expected =
        crate::services::mcp_bridge::config::validated_env_keys(connector.env_keys.as_deref())
            .map_err(|_| ())?;
    if !expected.iter().any(|item| item == env_key) {
        return Err(());
    }
    let key = crate::services::mcp_bridge::env_tokens::vault_key(connector_id, env_key);
    let secret = crate::services::api_keys::get_raw(&key).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

fn channel_secret(params: &Value) -> Result<CoreResponse, ()> {
    use crate::services::gateway::tokens::{self, GatewayTokenKind};
    let channel_id = string_param(params, "channelId")?;
    let account_id = string_param(params, "accountId")?;
    let kind =
        GatewayTokenKind::parse(channel_id, string_param(params, "kind")?).map_err(|_| ())?;
    let key = tokens::vault_key(channel_id, account_id, kind).map_err(|_| ())?;
    let secret = crate::services::api_keys::get_raw(&key).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

async fn call_mcp_tool(params: &Value) -> Result<CoreResponse, ()> {
    let connector_id = string_param(params, "connectorId")?;
    let tool_name = string_param(params, "toolName")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    super::validation::message(&arguments).map_err(|_| ())?;
    let (connector, tool) =
        crate::services::mcp_bridge::registry::resolve_enabled_tool(connector_id, tool_name)
            .await
            .map_err(|_| ())?;
    crate::services::mcp_bridge::arguments::validate(&arguments, tool.input_schema.as_ref())
        .map_err(|_| ())?;
    let result = tokio::time::timeout(
        Duration::from_millis(MCP_TOOL_TIMEOUT_MS as u64),
        connector.transport.call_tool(&tool.name, arguments),
    )
    .await
    .map_err(|_| ())?
    .map_err(|_| ())?;
    if result.is_error {
        return Err(());
    }
    Ok(CoreResponse::Json(Value::String(result.content)))
}

fn string_param<'a>(params: &'a Value, key: &str) -> Result<&'a str, ()> {
    let value = params.get(key).and_then(Value::as_str).ok_or(())?;
    super::validation::source_input(value).map_err(|_| ())?;
    Ok(value)
}

fn json_response(value: impl Serialize) -> Result<CoreResponse, ()> {
    serde_json::to_value(value)
        .map(CoreResponse::Json)
        .map_err(|_| ())
}

#[cfg(test)]
#[path = "core_bridge_tests.rs"]
mod tests;
