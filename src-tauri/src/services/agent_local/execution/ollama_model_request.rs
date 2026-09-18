use super::OllamaClient;
use crate::services::agent_local::{ollama_model_helpers, types_ollama::ModelInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OllamaModelError {
    Unavailable,
    NotFound,
    InvalidResponse,
}

impl OllamaClient {
    pub(crate) async fn inspect_model(&self, name: &str) -> Result<ModelInfo, OllamaModelError> {
        let base_url = self.base_url().await.map_err(|_| {
            ::log::warn!("[ollama] model_inspection_failed kind=Unavailable");
            OllamaModelError::Unavailable
        })?;
        self.inspect_model_at(&base_url, name).await
    }

    pub(super) async fn inspect_model_at(
        &self,
        base_url: &str,
        name: &str,
    ) -> Result<ModelInfo, OllamaModelError> {
        let result = self.request_model(base_url, name).await;
        if let Err(error) = result {
            // Classify the failure without recording URLs, response bodies or model names.
            ::log::warn!("[ollama] model_inspection_failed kind={error:?}");
        }
        result
    }

    async fn request_model(
        &self,
        base_url: &str,
        name: &str,
    ) -> Result<ModelInfo, OllamaModelError> {
        let response = self
            .client
            .post(format!("{base_url}/api/show"))
            .json(&serde_json::json!({"model": name}))
            .send()
            .await
            .map_err(|_| OllamaModelError::Unavailable)?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(OllamaModelError::NotFound);
        }
        if !response.status().is_success() {
            return Err(OllamaModelError::Unavailable);
        }
        let body = crate::services::secure_http::read_bounded(
            response,
            ollama_model_helpers::MAX_SHOW_RESPONSE_BYTES,
        )
        .await
        .map_err(|error| match error {
            crate::services::secure_http::SecureHttpError::Body => OllamaModelError::Unavailable,
            _ => OllamaModelError::InvalidResponse,
        })?;
        let json: serde_json::Value =
            serde_json::from_slice(&body).map_err(|_| OllamaModelError::InvalidResponse)?;
        ollama_model_helpers::parse_show_response(name, &json)
            .map_err(|_| OllamaModelError::InvalidResponse)
    }
}
