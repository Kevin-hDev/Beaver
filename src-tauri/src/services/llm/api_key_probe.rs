use super::route_profile::{ApiKeyHeader, AuthKind, AuthProbePolicy, EndpointPolicy};
use crate::services::secure_http::AuthenticatedClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProbeMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProbeAuth {
    Bearer,
    XApiKey,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProbeSpec {
    pub method: ProbeMethod,
    pub url: String,
    pub auth: ProbeAuth,
    pub headers: &'static [(&'static str, &'static str)],
    pub body: Option<serde_json::Value>,
}

pub(crate) fn resolve(provider_id: &str) -> Result<ProbeSpec, &'static str> {
    let profile =
        super::route_profile::find(provider_id).ok_or("provider_configuration_invalid")?;
    let (auth, headers) = match profile.auth {
        AuthKind::ApiKey {
            header, headers, ..
        } => (
            match header {
                ApiKeyHeader::Bearer => ProbeAuth::Bearer,
                ApiKeyHeader::XApiKey => ProbeAuth::XApiKey,
            },
            headers,
        ),
        AuthKind::OAuth { .. } | AuthKind::ClientOAuth { .. } | AuthKind::Local => {
            return Err("provider_configuration_invalid");
        }
    };
    let (base_url, models_endpoint) = match profile.endpoint {
        EndpointPolicy::Static {
            base_url,
            models_endpoint,
        } => (base_url.to_string(), models_endpoint.to_string()),
        EndpointPolicy::ProviderConnection { .. } => {
            let route =
                super::route::resolve(provider_id).ok_or("provider_configuration_invalid")?;
            (
                route.base_url.into_owned(),
                route.models_endpoint.into_owned(),
            )
        }
        EndpointPolicy::ConnectionConfigured
        | EndpointPolicy::OllamaLocal
        | EndpointPolicy::RegionAllowlist { .. }
        | EndpointPolicy::Workspace { .. }
        | EndpointPolicy::ValidatedHttps
        | EndpointPolicy::PinnedBackend { .. } => {
            return Err("provider_configuration_invalid");
        }
    };
    match profile.policies.auth_probe {
        AuthProbePolicy::ModelsGet if !models_endpoint.is_empty() => Ok(ProbeSpec {
            method: ProbeMethod::Get,
            url: format!("{base_url}{models_endpoint}"),
            auth,
            headers,
            body: None,
        }),
        AuthProbePolicy::ChatPing => Ok(ProbeSpec {
            method: ProbeMethod::Post,
            url: format!("{base_url}/chat/completions"),
            auth,
            headers,
            body: Some(serde_json::json!({
                "model": super::openai_compat::ping_model(provider_id),
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "hi"}],
            })),
        }),
        AuthProbePolicy::ModelsGet
        | AuthProbePolicy::OAuthCatalog
        | AuthProbePolicy::ClientNative
        | AuthProbePolicy::None => Err("provider_configuration_invalid"),
    }
}

pub(crate) fn request(
    client: &AuthenticatedClient,
    probe: &ProbeSpec,
    key: &str,
) -> reqwest::RequestBuilder {
    let mut request = match probe.method {
        ProbeMethod::Get => client.get(&probe.url),
        ProbeMethod::Post => client.post(&probe.url),
    };
    request = match probe.auth {
        ProbeAuth::Bearer => request.bearer_auth(key),
        ProbeAuth::XApiKey => request.header("x-api-key", key),
    };
    for (name, value) in probe.headers {
        request = request.header(*name, *value);
    }
    if let Some(body) = &probe.body {
        request = request.json(body);
    }
    request
}
