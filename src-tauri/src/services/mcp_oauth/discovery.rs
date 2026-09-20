use super::types::{AuthServerMetadata, ProtectedResourceMetadata};
use super::{issuer, network_guard, trusted_oauth};
use network_guard::DestinationRole;

pub async fn discover_auth_server(
    connector_id: &str,
    endpoint: &str,
) -> Result<AuthServerMetadata, String> {
    if !crate::services::mcp_bridge::trusted::is_trusted_endpoint_for_connector(
        connector_id,
        endpoint,
    ) {
        return Err("serveur MCP non autorisé".to_string());
    }
    if let Some(meta) = hardcoded_override(endpoint) {
        trusted_oauth::validate_metadata_endpoints(connector_id, &meta)?;
        return Ok(meta);
    }

    let expected_issuer = discover_issuer(connector_id, endpoint).await?;
    trusted_oauth::validate_issuer(connector_id, &expected_issuer)?;
    let meta = fetch_auth_server_metadata(connector_id, endpoint, &expected_issuer).await?;
    issuer::verify_metadata_issuer(&expected_issuer, &meta.issuer)?;
    trusted_oauth::validate_metadata_endpoints(connector_id, &meta)?;
    Ok(meta)
}

async fn discover_issuer(connector_id: &str, endpoint: &str) -> Result<String, String> {
    let client =
        network_guard::destination(connector_id, endpoint, DestinationRole::McpProbe, endpoint)
            .await?;
    let request = client
        .post()
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#);
    let resp = client
        .send(request)
        .await
        .map_err(|_| "impossible de contacter le serveur MCP".to_string())?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        if let Some(header) = resp
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(url) = extract_resource_metadata_url(header) {
                if let Some(issuer) =
                    fetch_issuer_from_resource_meta(connector_id, endpoint, &url).await?
                {
                    return Ok(issuer);
                }
            }
        }
    } else if !status.is_success() {
        return Err("sonde MCP refusée".to_string());
    }

    let base_url = endpoint_base_url(endpoint)?;

    let candidates = [
        format!("{base_url}/.well-known/oauth-protected-resource"),
        format!("{base_url}/.well-known/oauth-protected-resource/mcp"),
    ];

    for url in &candidates {
        if let Some(issuer) = fetch_issuer_from_resource_meta(connector_id, endpoint, url).await? {
            return Ok(issuer);
        }
    }

    Ok(base_url)
}

fn extract_resource_metadata_url(header: &str) -> Option<String> {
    if header.len() > 2048 {
        return None;
    }
    let mut found = None;
    for segment in header.split([',', ' ']) {
        let trimmed = segment.trim();
        if let Some(rest) = trimmed.strip_prefix("resource_metadata=") {
            let url = rest.trim_matches('"').trim();
            if found.is_some() {
                return None;
            }
            found = Some(url.to_string());
        }
    }
    found
}

async fn fetch_issuer_from_resource_meta(
    connector_id: &str,
    endpoint: &str,
    url: &str,
) -> Result<Option<String>, String> {
    let client = network_guard::destination(
        connector_id,
        endpoint,
        DestinationRole::ResourceMetadata,
        url,
    )
    .await?;
    let resp = client
        .send(client.get())
        .await
        .map_err(|_| "serveur non disponible".to_string())?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err("métadonnées OAuth refusées".to_string());
    }
    let meta: ProtectedResourceMetadata = super::bounded_json(resp).await?;
    let servers = meta
        .authorization_servers
        .ok_or("serveur d'autorisation absent".to_string())?;
    if servers.is_empty() || servers.len() > 8 {
        return Err("serveurs d'autorisation invalides".to_string());
    }
    for issuer in &servers {
        trusted_oauth::validate_issuer(connector_id, issuer)?;
    }
    Ok(servers.into_iter().next())
}

async fn fetch_auth_server_metadata(
    connector_id: &str,
    endpoint: &str,
    issuer: &str,
) -> Result<AuthServerMetadata, String> {
    let issuer_clean = issuer.trim_end_matches('/');
    let parsed = reqwest::Url::parse(issuer).map_err(|_| "émetteur OAuth invalide")?;
    let inserted = format!(
        "{}/.well-known/oauth-authorization-server{}",
        parsed.origin().ascii_serialization(),
        parsed.path().trim_end_matches('/')
    );
    let candidates = [
        inserted,
        format!("{issuer_clean}/.well-known/oauth-authorization-server"),
        format!("{issuer_clean}/.well-known/openid-configuration"),
    ];
    for url in &candidates {
        if let Some(meta) = try_fetch_metadata(connector_id, endpoint, url).await? {
            issuer::verify_metadata_issuer(issuer, &meta.issuer)?;
            return Ok(meta);
        }
    }

    Err("métadonnées du serveur d'autorisation non trouvées".to_string())
}

async fn try_fetch_metadata(
    connector_id: &str,
    endpoint: &str,
    url: &str,
) -> Result<Option<AuthServerMetadata>, String> {
    let client =
        network_guard::destination(connector_id, endpoint, DestinationRole::OAuth, url).await?;
    let resp = client
        .send(client.get())
        .await
        .map_err(|_| "serveur non disponible".to_string())?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err("métadonnées OAuth refusées".to_string());
    }
    super::bounded_json(resp).await.map(Some)
}

fn endpoint_base_url(endpoint: &str) -> Result<String, String> {
    let parsed = reqwest::Url::parse(endpoint).map_err(|_| "URL invalide".to_string())?;
    let host = parsed.host_str().unwrap_or("");
    let port = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();
    Ok(format!("{}://{host}{port}", parsed.scheme()))
}

fn hardcoded_override(endpoint: &str) -> Option<AuthServerMetadata> {
    let host = reqwest::Url::parse(endpoint).ok()?.host_str()?.to_string();

    if host == "api.githubcopilot.com" {
        return Some(AuthServerMetadata {
            issuer: "https://github.com/login/oauth".to_string(),
            authorization_response_iss_parameter_supported: false,
            authorization_endpoint: "https://github.com/login/oauth/authorize".to_string(),
            token_endpoint: "https://github.com/login/oauth/access_token".to_string(),
            registration_endpoint: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        });
    }

    if matches!(
        host.as_str(),
        "gmailmcp.googleapis.com" | "drivemcp.googleapis.com" | "calendarmcp.googleapis.com"
    ) {
        return Some(AuthServerMetadata {
            issuer: "https://accounts.google.com".to_string(),
            authorization_response_iss_parameter_supported: false,
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
            registration_endpoint: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        });
    }

    None
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
