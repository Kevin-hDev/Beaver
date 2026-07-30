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
) -> Result<Response, RequestError> {
    route
        .send_authenticated(client, purpose, |token, headers| {
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
