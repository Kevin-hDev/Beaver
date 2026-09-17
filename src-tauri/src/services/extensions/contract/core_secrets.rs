use super::core_bridge::CoreResponse;
use serde_json::Value;

pub(super) fn provider(params: &Value) -> Result<CoreResponse, ()> {
    let provider = super::core_bridge::string_param(params, "providerId")?;
    crate::services::api_keys::validate::validate_provider(provider).map_err(|_| ())?;
    let secret = crate::services::api_keys::get_key(provider).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

pub(super) async fn mcp_oauth(params: &Value) -> Result<CoreResponse, ()> {
    let connector = super::core_bridge::string_param(params, "connectorId")?;
    crate::services::mcp_bridge::config::validate_connector_id(connector).map_err(|_| ())?;
    let secret = crate::services::mcp_oauth::storage::get_valid_token(connector)
        .await
        .map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

pub(super) fn mcp_env(params: &Value) -> Result<CoreResponse, ()> {
    let connector_id = super::core_bridge::string_param(params, "connectorId")?;
    let env_key = super::core_bridge::string_param(params, "envKey")?;
    let connector = crate::services::mcp_bridge::config::find(connector_id)
        .map_err(|_| ())?
        .ok_or(())?;
    let expected =
        crate::services::mcp_bridge::config::validated_env_keys(connector.env_keys.as_deref())
            .map_err(|_| ())?;
    if !requested_resource(&expected, env_key) {
        return Err(());
    }
    let key = crate::services::mcp_bridge::env_tokens::vault_key(connector_id, env_key);
    let secret = crate::services::api_keys::get_raw(&key).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

pub(super) fn channel(params: &Value) -> Result<CoreResponse, ()> {
    use crate::services::gateway::tokens::{self, GatewayTokenKind};
    let channel_id = super::core_bridge::string_param(params, "channelId")?;
    let account_id = super::core_bridge::string_param(params, "accountId")?;
    let kind = GatewayTokenKind::parse(
        channel_id,
        super::core_bridge::string_param(params, "kind")?,
    )
    .map_err(|_| ())?;
    let key = tokens::vault_key(channel_id, account_id, kind).map_err(|_| ())?;
    let secret = crate::services::api_keys::get_raw(&key).map_err(|_| ())?;
    Ok(CoreResponse::Secret(secret))
}

pub(super) fn requested_resource(resources: &[String], requested: &str) -> bool {
    resources.iter().any(|item| item == requested)
}
