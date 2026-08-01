use std::time::Duration;

use reqwest::{Response, StatusCode};

use super::http_error;
use super::limits::{CONNECT_TIMEOUT, STREAM_STALL_TIMEOUT};
use super::types::CODEX_API_BASE;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;
use crate::services::llm::provider_error::ProviderErrorCode;
use crate::services::secure_http::{AuthenticatedClient, SecureHttpError};

#[derive(Clone, Copy)]
pub(super) enum RequestDeadline {
    Streaming,
    Total(Duration),
}

pub(super) async fn post(
    body: &str,
    model: &str,
    tool_count: usize,
    deadline: RequestDeadline,
) -> Result<Response, String> {
    let client = build_client(deadline)?;
    let credentials = token::ensure_valid().await?;
    let mut response = send_once(&client, &credentials, body).await?;
    if response.status() == StatusCode::UNAUTHORIZED {
        let refreshed = token::recover_after_unauthorized(credentials.access.as_str()).await?;
        drop(response);
        drop(credentials);
        response = send_once(&client, &refreshed, body).await?;
    }
    http_error::require_success(response, model, body.len(), tool_count).await
}

fn build_client(deadline: RequestDeadline) -> Result<AuthenticatedClient, String> {
    let result = match deadline {
        RequestDeadline::Streaming => {
            AuthenticatedClient::new_streaming(CONNECT_TIMEOUT, STREAM_STALL_TIMEOUT)
        }
        RequestDeadline::Total(timeout) => AuthenticatedClient::new(timeout),
    };
    result.map_err(|error| secure_http_error(error).to_string())
}

async fn send_once(
    client: &AuthenticatedClient,
    credentials: &CodexTokens,
    body: &str,
) -> Result<Response, String> {
    let request = client
        .post(format!("{CODEX_API_BASE}/responses"))
        .bearer_auth(credentials.access.as_str())
        .header("chatgpt-account-id", credentials.account_hint.as_str())
        .header("originator", crate::services::codex_oauth::ORIGINATOR)
        .header("User-Agent", crate::services::brand::user_agent())
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .body(body.to_string());
    client
        .send(request)
        .await
        .map_err(|error| secure_http_error(error).to_string())
}

fn secure_http_error(error: SecureHttpError) -> &'static str {
    match error {
        SecureHttpError::Configuration => ProviderErrorCode::ProviderConfigurationInvalid.as_str(),
        SecureHttpError::Status => ProviderErrorCode::ProviderRequestRejected.as_str(),
        _ => ProviderErrorCode::ProviderConnectionFailed.as_str(),
    }
}

#[cfg(test)]
#[path = "request_http_tests.rs"]
mod tests;
