pub async fn test_connector(connector: config::StoredConnector) -> Result<(), String> {
    config::validate_connector(&connector)?;
    token_validation::validate_connector_tokens(&connector).await?;
    let id = connector.id.clone();
    let enabled = build_connector(connector).ok_or("connecteur MCP invalide")?;
    invalidate_cache(&id);
    let tools = tokio::time::timeout(
        std::time::Duration::from_secs(TEST_TIMEOUT_SECS),
        enabled.transport.list_tools(),
    )
    .await
    .map_err(|_| "test MCP expiré".to_string())?
    .map_err(|_| "test MCP échoué".to_string())?;
    set_cached(&id, &tools);
    Ok(())
}

pub async fn test_connector_with_env(
    connector: config::StoredConnector,
    env_tokens: Vec<(String, zeroize::Zeroizing<String>)>,
) -> Result<(), String> {
    config::validate_connector(&connector)?;
    let command = config::install_command_for(&connector)
        .ok_or_else(|| "connecteur MCP invalide".to_string())?;
    let env_keys = config::validated_env_keys(connector.env_keys.as_deref())?;
    let id = connector.id.clone();
    process_manager::shutdown_one(&id);
    let transport = StdioTransport::new_with_env(id.clone(), command, env_keys, env_tokens);
    let result = run_probe(transport.list_tools()).await;
    process_manager::shutdown_one(&id);
    cache_probe_result(&id, &result);
    result.map(|_| ())
}

pub async fn test_connector_with_oauth_token(
    connector: config::StoredConnector,
    token: zeroize::Zeroizing<String>,
) -> Result<(), String> {
    config::validate_connector(&connector)?;
    let endpoint = connector
        .endpoint
        .clone()
        .ok_or_else(|| "connecteur MCP invalide".to_string())?;
    let id = connector.id.clone();
    let transport = HttpTransport::new_with_token(id.clone(), endpoint, token);
    let result = run_probe(transport.list_tools()).await;
    cache_probe_result(&id, &result);
    result.map(|_| ())
}

async fn run_probe<F>(probe: F) -> Result<Vec<McpToolDef>, String>
where
    F: std::future::Future<Output = Result<Vec<McpToolDef>, String>>,
{
    tokio::time::timeout(std::time::Duration::from_secs(TEST_TIMEOUT_SECS), probe)
        .await
        .map_err(|_| "test MCP expiré".to_string())?
        .map_err(|_| "test MCP échoué".to_string())
}

fn cache_probe_result(id: &str, result: &Result<Vec<McpToolDef>, String>) {
    if let Ok(tools) = result {
        set_cached(id, tools);
    } else {
        invalidate_cache(id);
    }
}
