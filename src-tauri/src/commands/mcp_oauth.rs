use crate::services::mcp_oauth::{flow, storage};

fn validate_connector_id(id: &str) -> Result<(), String> {
    crate::services::mcp_bridge::config::validate_connector_id(id)
}

#[tauri::command]
pub async fn start_mcp_oauth(
    app: tauri::AppHandle,
    connector_id: String,
    endpoint: String,
) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    if endpoint.is_empty() || !endpoint.starts_with("https://") {
        return Err("endpoint MCP non HTTPS".to_string());
    }
    if !crate::services::mcp_bridge::registry::is_trusted_endpoint_pub(&connector_id, &endpoint) {
        return Err("endpoint non autorisé pour OAuth".to_string());
    }
    tauri::async_runtime::spawn(flow::run(app, connector_id, endpoint));
    Ok(())
}

#[tauri::command]
pub async fn cancel_mcp_oauth(connector_id: String) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    flow::cancel(&connector_id);
    Ok(())
}

#[tauri::command]
pub async fn has_mcp_oauth_token(connector_id: String) -> Result<bool, String> {
    validate_connector_id(&connector_id)?;
    Ok(storage::has_tokens(&connector_id))
}

#[tauri::command]
pub async fn delete_mcp_oauth_token(connector_id: String) -> Result<(), String> {
    validate_connector_id(&connector_id)?;
    crate::services::mcp_bridge::registry::invalidate_cache(&connector_id);
    storage::delete_tokens(&connector_id)
}
