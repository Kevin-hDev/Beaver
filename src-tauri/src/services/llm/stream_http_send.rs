use reqwest::Response;

use super::request_purpose::RequestPurpose;
use super::route::{LlmRoute, RouteError};
use super::stream_http::RequestError;
use crate::services::secure_http::AuthenticatedClient;

pub(super) fn json_request_builder(
    client: &AuthenticatedClient,
    url: &str,
    payload: &serde_json::Value,
    token: &str,
    mut auth_headers: reqwest::header::HeaderMap,
    outbound_headers: &reqwest::header::HeaderMap,
) -> reqwest::RequestBuilder {
    auth_headers.extend(outbound_headers.clone());
    client
        .post(url)
        .headers(auth_headers)
        .bearer_auth(token)
        .json(payload)
}

pub async fn send_json_request(
    client: &AuthenticatedClient,
    route: &LlmRoute,
    url: &str,
    payload: &serde_json::Value,
    purpose: RequestPurpose,
    model: &str,
    session_id: Option<&str>,
) -> Result<Response, RequestError> {
    let outbound_headers = outbound_headers(route, model, session_id, purpose)?;
    route
        .send_authenticated(client, purpose, |token, headers| {
            json_request_builder(client, url, payload, token, headers, &outbound_headers)
        })
        .await
        .map_err(|error| match error {
            RouteError::Unauthorized if route.is_oauth() => {
                RequestError::Fatal("oauth_reauthentication_required".into())
            }
            RouteError::Unauthorized => RequestError::Fatal("auth_failed".into()),
            RouteError::Forbidden => RequestError::Fatal("provider_access_unavailable".into()),
            RouteError::Network => RequestError::Fatal(
                super::provider_error::ProviderErrorCode::ProviderConnectionFailed
                    .as_str()
                    .into(),
            ),
        })
}

pub(super) fn outbound_headers(
    route: &LlmRoute,
    model: &str,
    session_id: Option<&str>,
    purpose: RequestPurpose,
) -> Result<reqwest::header::HeaderMap, RequestError> {
    let mut headers =
        super::prompt_cache_policy::request_headers(route, Some(model), session_id, purpose)
            .map_err(|_| RequestError::InvalidConfiguration)?;
    if route.chat_provider_id == "xai-oauth" {
        headers.extend(
            crate::services::llm_oauth::xai_model_header(model)
                .map_err(|_| RequestError::InvalidConfiguration)?,
        );
    }
    Ok(headers)
}
