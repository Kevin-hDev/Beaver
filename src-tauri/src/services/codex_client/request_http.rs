use std::future::Future;
use std::time::Duration;

use reqwest::{Response, StatusCode};

use super::http_error;
use super::limits::{CONNECT_TIMEOUT, MODELS_TIMEOUT, STREAM_STALL_TIMEOUT};
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
    routing_hint: &str,
    model: &str,
    tool_count: usize,
    deadline: RequestDeadline,
) -> Result<Response, String> {
    #[cfg(test)]
    if let Some(response) =
        super::test_transport::dispatch_http(body, routing_hint, model, tool_count).await
    {
        return response;
    }
    let client = build_client(deadline)?;
    let credentials = token::ensure_valid().await?;
    let endpoint = format!("{CODEX_API_BASE}/responses");
    post_with_refresh(
        &client,
        &credentials,
        &endpoint,
        body,
        routing_hint,
        model,
        tool_count,
        token::recover_after_unauthorized(credentials.access.as_str()),
    )
    .await
}

#[expect(
    clippy::too_many_arguments,
    reason = "shared HTTP boundary keeps the body and validated routing capture explicit"
)]
pub(super) async fn post_with_refresh<F>(
    client: &AuthenticatedClient,
    credentials: &CodexTokens,
    endpoint: &str,
    body: &str,
    routing_hint: &str,
    model: &str,
    tool_count: usize,
    refresh: F,
) -> Result<Response, String>
where
    F: Future<Output = Result<CodexTokens, String>>,
{
    let mut response = send_once(client, credentials, endpoint, body, routing_hint).await?;
    if response.status() == StatusCode::UNAUTHORIZED {
        let refreshed = refresh.await?;
        drop(response);
        response = send_once(client, &refreshed, endpoint, body, routing_hint).await?;
    }
    http_error::require_success(response, model, body.len(), tool_count).await
}

pub(super) async fn get_models() -> Result<Response, String> {
    let client = build_client(RequestDeadline::Total(MODELS_TIMEOUT))?;
    let credentials = token::ensure_valid().await?;
    let mut response = send_models_once(&client, &credentials).await?;
    if response.status() == StatusCode::UNAUTHORIZED {
        let refreshed = token::recover_after_unauthorized(credentials.access.as_str()).await?;
        drop(response);
        drop(credentials);
        response = send_models_once(&client, &refreshed).await?;
    }
    if response.status().is_success() {
        Ok(response)
    } else {
        Err("model_catalog_unavailable".to_string())
    }
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
    endpoint: &str,
    body: &str,
    routing_hint: &str,
) -> Result<Response, String> {
    let request = client
        .post(endpoint)
        .bearer_auth(credentials.access.as_str())
        .header("chatgpt-account-id", credentials.account_hint.as_str())
        .header("originator", crate::services::codex_oauth::ORIGINATOR)
        .header("User-Agent", crate::services::brand::user_agent())
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .header("x-codex-routing-hint", routing_hint)
        .body(body.to_string());
    client
        .send(request)
        .await
        .map_err(|error| secure_http_error(error).to_string())
}

async fn send_models_once(
    client: &AuthenticatedClient,
    credentials: &CodexTokens,
) -> Result<Response, String> {
    let mut url = reqwest::Url::parse(&format!("{CODEX_API_BASE}/models"))
        .map_err(|_| "provider_configuration_invalid".to_string())?;
    url.query_pairs_mut()
        .append_pair("client_version", env!("CARGO_PKG_VERSION"));
    let request = client
        .get(url)
        .bearer_auth(credentials.access.as_str())
        .header("chatgpt-account-id", credentials.account_hint.as_str())
        .header("originator", crate::services::codex_oauth::ORIGINATOR)
        .header("User-Agent", crate::services::brand::user_agent())
        .header("Accept", "application/json");
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
