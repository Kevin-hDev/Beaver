//! Client unifié pour les fournisseurs OpenAI-compatibles, API ou OAuth natif.

use super::openai_compat_models;
use super::openai_compat_parsing::{map_error_status, parse_models_list};
use super::request_purpose::RequestPurpose;
use super::route::{self, LlmRoute, RouteError};
use super::types::{LlmError, ModelInfo};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, LLM_BODY_LIMIT};

pub struct OpenAiCompatProvider {
    pub(super) route: LlmRoute,
    pub(super) client: AuthenticatedClient,
}

pub fn ping_model(provider_id: &str) -> &'static str {
    openai_compat_models::ping_model(route::canonical_provider_id(provider_id))
}

impl OpenAiCompatProvider {
    pub fn new(provider_id: &str) -> Result<Self, LlmError> {
        let route = route::resolve(provider_id)
            .ok_or_else(|| LlmError::Provider("fournisseur inconnu".to_string()))?;
        let client = AuthenticatedClient::new(super::timeouts::request_timeout_for(provider_id))
            .map_err(|_| network_error())?;
        Ok(Self { route, client })
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        if self.route.chat_provider_id == "xai-oauth" {
            return crate::services::llm_oauth::xai_models().await;
        }
        let canonical = self.route.canonical_provider_id;
        if let Some(models) = openai_compat_models::static_model_infos(canonical) {
            return Ok(models);
        }
        let url = format!("{}{}", self.route.base_url, self.route.models_endpoint);
        let response = self
            .send(RequestPurpose::AccountMetadata, |token, headers| {
                self.client.get(&url).headers(headers).bearer_auth(token)
            })
            .await?;
        if !response.status().is_success() {
            return Err(map_error_status(response, self.route.error_policy).await);
        }
        let body = read_json(response).await?;
        if self.route.chat_provider_id == "moonshot-oauth" {
            super::kimi_models::parse_models_list(&body)
        } else {
            parse_models_list(&body, canonical)
        }
    }

    pub async fn test_connection(&self) -> Result<(), LlmError> {
        if openai_compat_models::has_static_models(self.route.canonical_provider_id) {
            return self.ping_chat().await;
        }
        self.list_models().await.map(|_| ())
    }

    async fn ping_chat(&self) -> Result<(), LlmError> {
        let url = format!("{}/chat/completions", self.route.base_url);
        let payload = serde_json::json!({
            "model": openai_compat_models::ping_model(self.route.canonical_provider_id),
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1,
        });
        let usage_generation =
            crate::services::provider_usage::credential_generation(self.route.chat_provider_id);
        let response = self
            .send(RequestPurpose::AccountMetadata, |token, headers| {
                self.client
                    .post(&url)
                    .headers(headers)
                    .bearer_auth(token)
                    .json(&payload)
            })
            .await?;
        crate::services::provider_usage::capture_headers(
            self.route.chat_provider_id,
            usage_generation,
            response.headers(),
        )
        .await;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(map_error_status(response, self.route.error_policy).await)
        }
    }

    pub(super) async fn send<F>(
        &self,
        purpose: RequestPurpose,
        build: F,
    ) -> Result<reqwest::Response, LlmError>
    where
        F: Fn(&str, reqwest::header::HeaderMap) -> reqwest::RequestBuilder,
    {
        let cache_policy = super::route_profile::cache_policy(self.route.chat_provider_id, "")
            .ok_or(LlmError::KnownProvider(
                super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
            ))?;
        let policy_headers =
            super::prompt_cache_policy::request_headers(cache_policy, None, purpose).map_err(
                |_| {
                    LlmError::KnownProvider(
                        super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
                    )
                },
            )?;
        self.route
            .send_authenticated(&self.client, purpose, |token, headers| {
                let mut headers = headers;
                headers.extend(policy_headers.clone());
                build(token, headers)
            })
            .await
            .map_err(|error| match error {
                RouteError::Unauthorized => LlmError::Unauthorized,
                RouteError::Forbidden => LlmError::Provider("usage non interactif refusé".into()),
                RouteError::Network => network_error(),
            })
    }
}

pub(super) async fn read_json(response: reqwest::Response) -> Result<serde_json::Value, LlmError> {
    read_json_bounded(response, LLM_BODY_LIMIT)
        .await
        .map_err(|_| LlmError::Parse("réponse invalide".to_string()))
}

fn network_error() -> LlmError {
    LlmError::Network("requête refusée".to_string())
}

#[cfg(test)]
#[path = "openai_compat_http_tests.rs"]
mod tests;
