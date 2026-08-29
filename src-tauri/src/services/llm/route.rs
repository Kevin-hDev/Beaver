use std::borrow::Cow;

use reqwest::{header::HeaderMap, RequestBuilder, Response};

use super::request_purpose::RequestPurpose;
use super::route_profile::{self, AuthKind, ClientSelector};
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
    #[cfg(test)]
    TestToken(&'static str),
}

#[derive(Debug, Clone)]
pub struct LlmRoute {
    pub chat_provider_id: &'static str,
    pub canonical_provider_id: &'static str,
    pub base_url: Cow<'static, str>,
    pub models_endpoint: Cow<'static, str>,
    pub display_name: &'static str,
    pub auto_max_tokens: bool,
    pub fallback_max_tokens: Option<u32>,
    pub usage_scope: UsageScope,
    pub(crate) error_policy: super::route_profile::ErrorPolicy,
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
                let response = send_oauth(
                    client,
                    provider,
                    purpose,
                    &token.value,
                    token.user_id.as_ref().map(|value| value.as_str()),
                    &build,
                )
                .await?;
                if oauth_401_action(response.status().as_u16(), false) != OAuth401Action::Refresh {
                    return Ok(response);
                }
                let refreshed = llm_oauth::force_refresh(provider, token.generation)
                    .await
                    .map_err(|_| RouteError::Unauthorized)?;
                let response = send_oauth(
                    client,
                    provider,
                    purpose,
                    &refreshed.value,
                    refreshed.user_id.as_ref().map(|value| value.as_str()),
                    &build,
                )
                .await?;
                if oauth_401_action(response.status().as_u16(), true) == OAuth401Action::Invalidate
                {
                    llm_oauth::invalidate(provider).await;
                }
                Ok(response)
            }
            #[cfg(test)]
            AuthSource::TestToken(token) => client
                .send(build(token, HeaderMap::new()))
                .await
                .map_err(|_| RouteError::Network),
        }
    }

    pub const fn is_oauth(&self) -> bool {
        matches!(self.auth_source, AuthSource::OAuth(_))
    }

    fn permits(&self, purpose: RequestPurpose) -> bool {
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
    user_id: Option<&str>,
    build: &F,
) -> Result<Response, RouteError>
where
    F: Fn(&str, HeaderMap) -> RequestBuilder,
{
    let headers = llm_oauth::request_headers_with_identity(provider, purpose, user_id)
        .map_err(|_| RouteError::Network)?;
    client
        .send(build(token, headers))
        .await
        .map_err(|_| RouteError::Network)
}

pub fn resolve(provider_id: &str) -> Option<LlmRoute> {
    let profile = route_profile::find(provider_id)?;
    if matches!(
        profile.client,
        ClientSelector::Codex | ClientSelector::OllamaLocal
    ) {
        return None;
    }
    let (base_url, models_endpoint) = match profile.endpoint {
        route_profile::EndpointPolicy::Static {
            base_url,
            models_endpoint,
        } => (Cow::Borrowed(base_url), Cow::Borrowed(models_endpoint)),
        route_profile::EndpointPolicy::ProviderConnection {
            resolver: route_profile::ConnectionEndpointResolver::QwenModelStudio,
        } => {
            let endpoint =
                match crate::services::provider_connections::qwen::load_resolved_endpoint() {
                    Ok(Some(endpoint)) => endpoint,
                    Ok(None) => return None,
                    Err(_) => {
                        log::warn!("provider=qwen event=route_hidden reason=invalid_connection");
                        return None;
                    }
                };
            (Cow::Owned(endpoint.base_url), Cow::Borrowed("/models"))
        }
        route_profile::EndpointPolicy::ConnectionConfigured
        | route_profile::EndpointPolicy::OllamaLocal
        | route_profile::EndpointPolicy::RegionAllowlist { .. }
        | route_profile::EndpointPolicy::Workspace { .. }
        | route_profile::EndpointPolicy::ValidatedHttps
        | route_profile::EndpointPolicy::PinnedBackend { .. } => return None,
    };
    let auth_source = match profile.auth {
        AuthKind::ApiKey { credential_id, .. } => AuthSource::ApiKey(credential_id),
        AuthKind::OAuth { provider, .. } => AuthSource::OAuth(provider),
        AuthKind::ClientOAuth { .. } | AuthKind::Local => return None,
    };
    Some(LlmRoute {
        chat_provider_id: profile.id.provider_id(),
        canonical_provider_id: profile.canonical_provider.as_str(),
        base_url,
        models_endpoint,
        display_name: profile.display_name,
        auto_max_tokens: profile.output_limits.automatic,
        fallback_max_tokens: profile.output_limits.fallback,
        usage_scope: if profile.availability.silent {
            UsageScope::Any
        } else {
            UsageScope::InteractiveOnly
        },
        error_policy: profile.policies.errors,
        auth_source,
    })
}

pub fn canonical_provider_id(provider_id: &str) -> &str {
    route_profile::find(provider_id)
        .map(|profile| profile.canonical_provider.as_str())
        .unwrap_or(provider_id)
}

#[cfg(test)]
#[path = "route_test_support.rs"]
mod test_support;
#[cfg(test)]
pub(super) fn test_route(chat_provider_id: &'static str) -> LlmRoute {
    test_support::test_route(chat_provider_id)
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
