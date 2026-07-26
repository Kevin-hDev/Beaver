use reqwest::{header::HeaderMap, RequestBuilder, Response};

use super::catalog;
use super::request_purpose::RequestPurpose;
use crate::services::llm_oauth::{self, LlmOAuthProvider};
use crate::services::secure_http::AuthenticatedClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageScope {
    Any,
    InteractiveOnly,
}

#[derive(Debug, Clone, Copy)]
enum AuthSource {
    ApiKey(&'static str),
    OAuth(LlmOAuthProvider),
}

#[derive(Debug, Clone, Copy)]
pub struct LlmRoute {
    pub chat_provider_id: &'static str,
    pub canonical_provider_id: &'static str,
    pub base_url: &'static str,
    pub models_endpoint: &'static str,
    pub display_name: &'static str,
    pub default_max_tokens: Option<u32>,
    pub usage_scope: UsageScope,
    auth_source: AuthSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    Unauthorized,
    Forbidden,
    Network,
}

impl LlmRoute {
    pub async fn send_authenticated<F>(
        &self,
        client: &AuthenticatedClient,
        purpose: RequestPurpose,
        build: F,
    ) -> Result<Response, RouteError>
    where
        F: Fn(&str, HeaderMap) -> RequestBuilder,
    {
        if !self.permits(purpose) {
            return Err(RouteError::Forbidden);
        }
        match self.auth_source {
            AuthSource::ApiKey(provider_id) => {
                let key = crate::services::api_keys::get_key(provider_id)
                    .map_err(|_| RouteError::Unauthorized)?;
                client
                    .send(build(&key, HeaderMap::new()))
                    .await
                    .map_err(|_| RouteError::Network)
            }
            AuthSource::OAuth(provider) => {
                let token = llm_oauth::access_token(provider)
                    .await
                    .map_err(|_| RouteError::Unauthorized)?;
                let response = send_oauth(client, provider, purpose, &token.value, &build).await?;
                if oauth_401_action(response.status().as_u16(), false) != OAuth401Action::Refresh {
                    return Ok(response);
                }
                let refreshed = llm_oauth::force_refresh(provider, token.generation)
                    .await
                    .map_err(|_| RouteError::Unauthorized)?;
                let response =
                    send_oauth(client, provider, purpose, &refreshed.value, &build).await?;
                if oauth_401_action(response.status().as_u16(), true) == OAuth401Action::Invalidate
                {
                    llm_oauth::invalidate(provider).await;
                }
                Ok(response)
            }
        }
    }

    pub const fn is_oauth(self) -> bool {
        matches!(self.auth_source, AuthSource::OAuth(_))
    }

    fn permits(self, purpose: RequestPurpose) -> bool {
        self.usage_scope == UsageScope::Any || purpose.allows_interactive_oauth()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OAuth401Action {
    None,
    Refresh,
    Invalidate,
}

fn oauth_401_action(status: u16, already_refreshed: bool) -> OAuth401Action {
    match (status, already_refreshed) {
        (401, false) => OAuth401Action::Refresh,
        (401, true) => OAuth401Action::Invalidate,
        _ => OAuth401Action::None,
    }
}

async fn send_oauth<F>(
    client: &AuthenticatedClient,
    provider: LlmOAuthProvider,
    purpose: RequestPurpose,
    token: &str,
    build: &F,
) -> Result<Response, RouteError>
where
    F: Fn(&str, HeaderMap) -> RequestBuilder,
{
    let headers =
        llm_oauth::request_headers_for(provider, purpose).map_err(|_| RouteError::Network)?;
    client
        .send(build(token, headers))
        .await
        .map_err(|_| RouteError::Network)
}

pub fn resolve(provider_id: &str) -> Option<LlmRoute> {
    match provider_id {
        "xai-oauth" => Some(oauth_route(
            "xai-oauth",
            "xai",
            "https://api.x.ai/v1",
            "",
            "xAI",
            LlmOAuthProvider::Xai,
        )),
        "moonshot-oauth" => Some(oauth_route(
            "moonshot-oauth",
            "moonshot",
            "https://api.kimi.com/coding/v1",
            "/models",
            "Moonshot AI",
            LlmOAuthProvider::Kimi,
        )),
        _ => catalog::find(provider_id).map(|spec| LlmRoute {
            chat_provider_id: spec.id,
            canonical_provider_id: spec.id,
            base_url: spec.base_url,
            models_endpoint: spec.models_endpoint,
            display_name: spec.display_name,
            default_max_tokens: spec.default_max_tokens,
            usage_scope: UsageScope::Any,
            auth_source: AuthSource::ApiKey(spec.id),
        }),
    }
}

pub fn canonical_provider_id(provider_id: &str) -> &str {
    match provider_id {
        "xai-oauth" => "xai",
        "moonshot-oauth" => "moonshot",
        _ => provider_id,
    }
}

pub fn is_interactive_only(provider_id: &str) -> bool {
    resolve(provider_id).is_some_and(|route| route.usage_scope == UsageScope::InteractiveOnly)
}

fn oauth_route(
    chat_provider_id: &'static str,
    canonical_provider_id: &'static str,
    base_url: &'static str,
    models_endpoint: &'static str,
    display_name: &'static str,
    provider: LlmOAuthProvider,
) -> LlmRoute {
    LlmRoute {
        chat_provider_id,
        canonical_provider_id,
        base_url,
        models_endpoint,
        display_name,
        default_max_tokens: Some(64_000),
        usage_scope: UsageScope::InteractiveOnly,
        auth_source: AuthSource::OAuth(provider),
    }
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
