use crate::services::mcp_oauth::{flow, storage};

fn validate_connector_id(id: &str) -> Result<(), String> {
    crate::services::mcp_bridge::config::validate_connector_id(id)
}

fn validate_start(connector_id: &str, endpoint: &str) -> Result<(), String> {
    validate_connector_id(connector_id)?;
    if connector_id == "slack" {
        return Err("connexion MCP indisponible".to_string());
    }
    if endpoint.is_empty() || !endpoint.starts_with("https://") {
        return Err("endpoint MCP non HTTPS".to_string());
    }
    if !crate::services::mcp_bridge::registry::is_trusted_endpoint_pub(connector_id, endpoint) {
        return Err("endpoint non autorisé pour OAuth".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn start_mcp_oauth(
    app: tauri::AppHandle,
    work: tauri::State<'_, crate::services::oauth_work::OAuthWorkServices>,
    connector_id: String,
    endpoint: String,
) -> Result<(), String> {
    validate_start(&connector_id, &endpoint)?;
    work.spawn(move |cancel| flow::run(app, connector_id, endpoint, cancel))
        .map_err(|_| "Connexion impossible".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn slack_oauth_cannot_start_without_app_identity() {
        assert!(super::validate_start("slack", "https://mcp.slack.com/mcp").is_err());
        assert!(super::validate_start("lucid", "https://mcp.lucid.app/mcp").is_ok());
    }
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
    crate::services::mcp_bridge::registry::mutate_identity(&connector_id, |_| {
        storage::delete_tokens(&connector_id)
    })
}
