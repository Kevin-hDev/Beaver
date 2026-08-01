use reqwest::Response;

use super::request_purpose::RequestPurpose;
use super::route::{LlmRoute, RouteError};
use super::stream_http::RequestError;
use crate::services::secure_http::AuthenticatedClient;

pub async fn send_json_request(
    client: &AuthenticatedClient,
    route: &LlmRoute,
    url: &str,
    payload: &serde_json::Value,
    purpose: RequestPurpose,
    model: &str,
    session_id: Option<&str>,
) -> Result<Response, RequestError> {
    let cache_headers =
        super::prompt_cache_policy::request_headers(route, Some(model), session_id, purpose)
            .map_err(|_| RequestError::InvalidConfiguration)?;
    route
        .send_authenticated(client, purpose, |token, headers| {
            let mut headers = headers;
            headers.extend(cache_headers.clone());
            client
                .post(url)
                .headers(headers)
                .bearer_auth(token)
                .json(payload)
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
