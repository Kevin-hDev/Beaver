use reqwest::{RequestBuilder, Response};

use super::{AuthSource, LlmRoute, OAuth401Action, RouteError};
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::secure_http::AuthenticatedClient;

pub(super) async fn send<F>(
    route: &LlmRoute,
    client: &AuthenticatedClient,
    purpose: RequestPurpose,
    fixture_payload: Option<&serde_json::Value>,
    build: F,
) -> Result<Response, RouteError>
where
    F: Fn(&str, reqwest::header::HeaderMap) -> RequestBuilder,
{
    if !route.permits(purpose) {
        return Err(RouteError::Forbidden);
    }
    match route.auth_source {
        AuthSource::ApiKey(provider_id) => {
            let key = crate::services::api_keys::get_key(provider_id)
                .map_err(|_| RouteError::Unauthorized)?;
            send_once(client, fixture_payload, build(&key, Default::default())).await
        }
        AuthSource::OAuth(provider) => {
            let token = crate::services::llm_oauth::access_token(provider)
                .await
                .map_err(|_| RouteError::Unauthorized)?;
            let response = send_oauth(
                client,
                provider,
                purpose,
                &token.value,
                token.user_id.as_ref().map(|value| value.as_str()),
                fixture_payload,
                &build,
            )
            .await?;
            if super::oauth_401_action(response.status().as_u16(), false) != OAuth401Action::Refresh
            {
                return Ok(response);
            }
            let refreshed = crate::services::llm_oauth::force_refresh(provider, token.generation)
                .await
                .map_err(|_| RouteError::Unauthorized)?;
            let response = send_oauth(
                client,
                provider,
                purpose,
                &refreshed.value,
                refreshed.user_id.as_ref().map(|value| value.as_str()),
                fixture_payload,
                &build,
            )
            .await?;
            if super::oauth_401_action(response.status().as_u16(), true)
                == OAuth401Action::Invalidate
            {
                crate::services::llm_oauth::invalidate(provider).await;
            }
            Ok(response)
        }
        #[cfg(test)]
        AuthSource::TestToken(token) => {
            send_once(client, fixture_payload, build(token, Default::default())).await
        }
        #[cfg(test)]
        AuthSource::TestOAuth(token) => {
            let response =
                send_once(client, fixture_payload, build(token, Default::default())).await?;
            if response.status().as_u16() != 401 {
                return Ok(response);
            }
            send_once(client, fixture_payload, build(token, Default::default())).await
        }
    }
}

async fn send_oauth<F>(
    client: &AuthenticatedClient,
    provider: crate::services::llm_oauth::LlmOAuthProvider,
    purpose: RequestPurpose,
    token: &str,
    user_id: Option<&str>,
    fixture_payload: Option<&serde_json::Value>,
    build: &F,
) -> Result<Response, RouteError>
where
    F: Fn(&str, reqwest::header::HeaderMap) -> RequestBuilder,
{
    let headers =
        crate::services::llm_oauth::request_headers_with_identity(provider, purpose, user_id)
            .map_err(|_| RouteError::Network)?;
    send_once(client, fixture_payload, build(token, headers)).await
}

async fn send_once(
    client: &AuthenticatedClient,
    fixture_payload: Option<&serde_json::Value>,
    request: RequestBuilder,
) -> Result<Response, RouteError> {
    #[cfg(debug_assertions)]
    if let Some(payload) = fixture_payload {
        crate::services::reasoning_fixture_budget::authorize_payload(payload)
            .map_err(RouteError::FixtureBudget)?;
    }
    #[cfg(not(debug_assertions))]
    let _ = fixture_payload;
    client.send(request).await.map_err(|_| RouteError::Network)
}
