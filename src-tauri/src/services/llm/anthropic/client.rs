use reqwest::header::{HeaderName, HeaderValue};

use super::models;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::route::{self, RouteError};
use crate::services::llm::route_profile::{ApiKeyHeader, AuthKind};
use crate::services::llm::types::{LlmError, ModelInfo};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, LLM_BODY_LIMIT};

pub(super) type StaticHeaders = &'static [(&'static str, &'static str)];

pub(in crate::services::llm) async fn list_models() -> Result<Vec<ModelInfo>, LlmError> {
    models::resolve_catalog(fetch_models().await)
}

pub(in crate::services::llm) async fn test_connection() -> Result<(), LlmError> {
    fetch_models().await.map(|_| ())
}

async fn fetch_models() -> Result<Vec<ModelInfo>, LlmError> {
    let route = route::resolve("anthropic").ok_or_else(configuration_error)?;
    let client = AuthenticatedClient::new(super::super::timeouts::request_timeout_for("anthropic"))
        .map_err(|_| network_error())?;
    let url = format!("{}{}", route.base_url, route.models_endpoint);
    let (header, static_headers) = auth_headers()?;
    let response = route
        .send_authenticated(
            &client,
            RequestPurpose::AccountMetadata,
            |token, inherited| {
                let request = client
                    .get(&url)
                    .query(&[("limit", "500")])
                    .headers(inherited);
                let request = match header {
                    ApiKeyHeader::XApiKey => request.header("x-api-key", token),
                    ApiKeyHeader::Bearer => request.bearer_auth(token),
                };
                static_headers
                    .iter()
                    .fold(request, |request, (name, value)| {
                        request.header(*name, *value)
                    })
            },
        )
        .await
        .map_err(map_route_error)?;
    if !response.status().is_success() {
        return Err(super::super::openai_compat_parsing::map_error_status(
            response,
            route.error_policy,
        )
        .await);
    }
    let body = read_json_bounded(response, LLM_BODY_LIMIT)
        .await
        .map_err(|_| {
            LlmError::KnownProvider(
                crate::services::llm::provider_error::ProviderErrorCode::ModelCatalogUnavailable,
            )
        })?;
    models::parse_catalog(&body)
}

pub(super) fn auth_headers() -> Result<(ApiKeyHeader, StaticHeaders), LlmError> {
    let profile = super::super::route_profile::find("anthropic").ok_or_else(configuration_error)?;
    let AuthKind::ApiKey {
        header, headers, ..
    } = profile.auth
    else {
        return Err(configuration_error());
    };
    for (name, value) in headers {
        HeaderName::from_bytes(name.as_bytes()).map_err(|_| configuration_error())?;
        HeaderValue::from_str(value).map_err(|_| configuration_error())?;
    }
    Ok((header, headers))
}

fn map_route_error(error: RouteError) -> LlmError {
    match error {
        RouteError::Unauthorized => LlmError::Unauthorized,
        RouteError::Forbidden => configuration_error(),
        RouteError::Network => network_error(),
    }
}

fn configuration_error() -> LlmError {
    LlmError::KnownProvider(
        crate::services::llm::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
    )
}

fn network_error() -> LlmError {
    LlmError::Network("requête refusée".into())
}
