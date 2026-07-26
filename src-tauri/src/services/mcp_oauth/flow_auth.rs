use std::time::Duration;

use zeroize::Zeroizing;

use super::types::{DcrResponse, OAuthTokens, TokenResponse};
use crate::services::secure_http::AuthenticatedClient;

pub fn build_auth_url(
    auth_endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
    resource: &str,
    scopes: Option<&str>,
) -> Result<String, String> {
    let mut url = reqwest::Url::parse(auth_endpoint)
        .map_err(|_| "URL d'autorisation invalide".to_string())?;
    if url.scheme() != "https" {
        return Err("URL d'autorisation non HTTPS".to_string());
    }
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state)
        .append_pair("resource", resource);
    if let Some(s) = scopes {
        url.query_pairs_mut().append_pair("scope", s);
    }
    Ok(url.to_string())
}

pub fn open_browser(_app: &tauri::AppHandle, url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("URL d'autorisation non HTTPS".to_string());
    }
    open_url_native(url)
}

#[cfg(target_os = "macos")]
fn open_url_native(url: &str) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|_| "impossible d'ouvrir le navigateur".to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_url_native(url: &str) -> Result<(), String> {
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map_err(|_| "impossible d'ouvrir le navigateur".to_string())?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_url_native(url: &str) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|_| "impossible d'ouvrir le navigateur".to_string())?;
    Ok(())
}

pub fn verify_state_constant_time(expected: &str, received: &str) -> Result<(), String> {
    const STATE_LEN: usize = 43;
    let mut expected_fixed = [0_u8; STATE_LEN];
    let mut received_fixed = [0_u8; STATE_LEN];
    let mut diff = copy_fixed_state(expected, &mut expected_fixed)
        | copy_fixed_state(received, &mut received_fixed);
    for index in 0..STATE_LEN {
        diff |= expected_fixed[index] ^ received_fixed[index];
    }
    if diff != 0 {
        return Err("état OAuth invalide".to_string());
    }
    Ok(())
}

fn copy_fixed_state(input: &str, output: &mut [u8; 43]) -> u8 {
    let bytes = input.as_bytes();
    let valid = u8::from(
        bytes.len() == output.len()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_'),
    );
    for (index, byte) in bytes.iter().take(output.len()).enumerate() {
        output[index] = *byte;
    }
    valid ^ 1
}

pub async fn register_client(
    connector_id: &str,
    registration_url: &str,
    redirect_uri: &str,
) -> Result<String, String> {
    super::trusted_oauth::validate_endpoint(connector_id, registration_url)?;
    let client = AuthenticatedClient::new(Duration::from_secs(15))
        .map_err(|_| "erreur interne".to_string())?;

    let body = serde_json::json!({
        "client_name": crate::services::mcp_bridge::identity::registration_name(connector_id),
        "application_type": "native",
        "redirect_uris": [redirect_uri],
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "token_endpoint_auth_method": "none",
    });

    let request = client.post(registration_url).json(&body);
    let resp = client
        .send_success(request)
        .await
        .map_err(|_| "échec de l'enregistrement du client".to_string())?;

    let dcr: DcrResponse = super::bounded_json(resp).await?;

    Ok(dcr.client_id.clone())
}

pub async fn exchange_code(
    connector_id: &str,
    token_endpoint: &str,
    code: &str,
    client_id: &str,
    client_secret: Option<&str>,
    verifier: &Zeroizing<String>,
    redirect_uri: &str,
    resource: &str,
) -> Result<OAuthTokens, String> {
    super::trusted_oauth::validate_endpoint(connector_id, token_endpoint)?;

    let client = AuthenticatedClient::new(Duration::from_secs(15))
        .map_err(|_| "erreur interne".to_string())?;

    let mut params: Vec<(&str, &str)> = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("code_verifier", verifier.as_str()),
        ("redirect_uri", redirect_uri),
        ("resource", resource),
    ];
    if let Some(secret) = client_secret {
        params.push(("client_secret", secret));
    }

    let request = client
        .post(token_endpoint)
        .header("Accept", "application/json")
        .form(&params);
    let resp = client
        .send_success(request)
        .await
        .map_err(|_| "échec de l'échange du code".to_string())?;

    let mut raw: TokenResponse = super::bounded_json(resp).await?;

    if raw.access_token.is_empty() {
        return Err("échec de l'authentification".to_string());
    }

    Ok(OAuthTokens::from_response(
        &mut raw,
        token_endpoint,
        client_id,
        client_secret,
    ))
}

#[cfg(test)]
#[path = "flow_auth_tests.rs"]
mod tests;
