use std::sync::LazyLock;
use std::time::Duration;

use reqwest::StatusCode;
use serde::Serialize;
use tokio::sync::Semaphore;
use zeroize::Zeroizing;

use super::store::CodexTokens;
use super::token_response::{self, CodexTokenResponse};
use super::{store, CLIENT_ID};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, OAUTH_BODY_LIMIT};

const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
static TOKEN_LOCK: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<CodexTokens, String> {
    let body = Zeroizing::new(format!(
        "grant_type=authorization_code&client_id={CLIENT_ID}&code={}&code_verifier={}&redirect_uri={}",
        urlencoding::encode(code),
        urlencoding::encode(code_verifier),
        urlencoding::encode(redirect_uri),
    ));
    token_response::from_exchange(post_body(&body, "application/x-www-form-urlencoded").await?)
        .map_err(|_| invalid_response())
}

pub async fn ensure_valid() -> Result<CodexTokens, String> {
    let observed = store::load()?.ok_or_else(not_connected)?;
    if !observed.needs_refresh() {
        return Ok(observed);
    }
    match refresh_due(&observed).await {
        Ok(refreshed) => Ok(refreshed),
        Err(error) if can_use_still_valid_token(&error, &observed) => Ok(observed),
        Err(error) => Err(error),
    }
}

pub async fn recover_after_unauthorized(rejected_access: &str) -> Result<CodexTokens, String> {
    let _guard = acquire_token_lock().await?;
    let current = store::load()?.ok_or_else(not_connected)?;
    if !constant_time_secret_eq(current.access.as_bytes(), rejected_access.as_bytes()) {
        return Ok(current);
    }
    refresh_and_save(&current).await
}

async fn refresh_due(observed: &CodexTokens) -> Result<CodexTokens, String> {
    let _guard = acquire_token_lock().await?;
    let current = store::load()?.ok_or_else(not_connected)?;
    if !constant_time_secret_eq(current.access.as_bytes(), observed.access.as_bytes())
        || !current.needs_refresh()
    {
        return Ok(current);
    }
    refresh_and_save(&current).await
}

pub async fn save_login(tokens: &CodexTokens) -> Result<(), String> {
    let _guard = acquire_token_lock().await?;
    store::save(tokens)
}

pub async fn clear_login() -> Result<(), String> {
    let _guard = acquire_token_lock().await?;
    store::clear()
}

async fn refresh_and_save(current: &CodexTokens) -> Result<CodexTokens, String> {
    let body = refresh_body(current.refresh.as_str())?;
    let response = post_body(&body, "application/json").await;
    let refreshed =
        token_response::from_refresh(response?, current).map_err(|_| invalid_response())?;
    store::save(&refreshed)?;
    Ok(refreshed)
}

fn refresh_body(refresh_token: &str) -> Result<Zeroizing<String>, String> {
    #[derive(Serialize)]
    struct RefreshRequest<'a> {
        client_id: &'a str,
        grant_type: &'static str,
        refresh_token: &'a str,
    }
    serde_json::to_string(&RefreshRequest {
        client_id: CLIENT_ID,
        grant_type: "refresh_token",
        refresh_token,
    })
    .map(Zeroizing::new)
    .map_err(|_| invalid_response())
}

async fn post_body(body: &str, content_type: &'static str) -> Result<CodexTokenResponse, String> {
    let client = AuthenticatedClient::new(Duration::from_secs(15))
        .map_err(|_| "provider_configuration_invalid".to_string())?;
    let request = client
        .post(TOKEN_URL)
        .header("Content-Type", content_type)
        .body(body.to_string());
    let response = client
        .send(request)
        .await
        .map_err(|_| "provider_connection_failed".to_string())?;
    if response.status().is_server_error()
        || matches!(
            response.status(),
            StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS
        )
    {
        return Err("provider_temporarily_unavailable".to_string());
    }
    if !response.status().is_success() {
        return Err("oauth_reauthentication_required".to_string());
    }
    read_json_bounded(response, OAUTH_BODY_LIMIT)
        .await
        .map_err(|_| invalid_response())
}

fn can_use_still_valid_token(error: &str, current: &CodexTokens) -> bool {
    !current.is_expired()
        && matches!(
            error,
            "provider_connection_failed" | "provider_temporarily_unavailable"
        )
}

async fn acquire_token_lock() -> Result<tokio::sync::SemaphorePermit<'static>, String> {
    TOKEN_LOCK
        .acquire()
        .await
        .map_err(|_| "oauth_reauthentication_required".to_string())
}

pub(crate) fn constant_time_secret_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        let left_byte = left.get(index).copied().unwrap_or_default();
        let right_byte = right.get(index).copied().unwrap_or_default();
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

fn invalid_response() -> String {
    "oauth_reauthentication_required".to_string()
}

fn not_connected() -> String {
    "oauth_reauthentication_required".to_string()
}

#[cfg(test)]
#[path = "token_tests.rs"]
mod tests;
