use crate::services::llm::provider_error::{catalog_code, ProviderErrorCode};
use crate::services::llm::types::ModelInfo;
use crate::services::oauth_providers::{self, ProviderId};
use serde::Serialize;

const MAX_OAUTH_MODELS: usize = 600;
const MAX_OAUTH_ISSUES: usize = 3;

#[derive(Serialize)]
pub struct OAuthProviderModel {
    pub id: String,
    pub provider_id: ProviderId,
    pub connection_id: &'static str,
    pub provider_display_name: &'static str,
    pub display_name: String,
    pub context_length: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub supports_fast_mode: bool,
    pub reasoning_modes: Vec<String>,
    pub default_reasoning_mode: Option<String>,
    pub context_usage_includes_reasoning: bool,
    pub interactive_only: bool,
}

#[derive(Serialize)]
pub struct OAuthProviderModelIssue {
    pub provider_id: ProviderId,
    pub code: ProviderErrorCode,
}

#[derive(Default, Serialize)]
pub struct OAuthProviderModelsResponse {
    pub models: Vec<OAuthProviderModel>,
    pub issues: Vec<OAuthProviderModelIssue>,
}

#[tauri::command]
pub async fn list_oauth_provider_models() -> OAuthProviderModelsResponse {
    let statuses = oauth_providers::list_statuses();
    let mut response = OAuthProviderModelsResponse::default();
    add_codex_models(&statuses, &mut response).await;
    add_external_models(&statuses, ProviderId::Xai, &mut response).await;
    add_external_models(&statuses, ProviderId::Moonshot, &mut response).await;
    response.models.truncate(MAX_OAUTH_MODELS);
    response.issues.truncate(MAX_OAUTH_ISSUES);
    response
}

async fn add_codex_models(
    statuses: &[oauth_providers::OAuthProviderStatus],
    response: &mut OAuthProviderModelsResponse,
) {
    if !connected(statuses, ProviderId::OpenAi) {
        return;
    }
    let models = match crate::commands::codex::resolved_codex_models().await {
        Ok(models) => models,
        Err(error) => {
            let code = if error == ProviderErrorCode::OAuthReauthenticationRequired.as_str() {
                ProviderErrorCode::OAuthReauthenticationRequired
            } else {
                ProviderErrorCode::ModelCatalogUnavailable
            };
            response.issues.push(OAuthProviderModelIssue {
                provider_id: ProviderId::OpenAi,
                code,
            });
            crate::services::codex_client::model_catalog::fallback_models()
        }
    };
    crate::services::llm::runtime_models::replace_provider("codex-oauth", &models);
    response.models.extend(models.into_iter().map(|model| {
        let display_name = model
            .display_name
            .clone()
            .unwrap_or_else(|| model.id.clone());
        OAuthProviderModel {
            display_name,
            id: model.id,
            provider_id: ProviderId::OpenAi,
            connection_id: ProviderId::OpenAi.usage_connection_id(),
            provider_display_name: ProviderId::OpenAi.display_name(),
            context_length: model.context_length,
            supports_tools: model.supports_tools,
            supports_vision: model.supports_vision,
            supports_thinking: model.supports_thinking,
            supports_fast_mode: model.supports_fast_mode,
            reasoning_modes: model.reasoning_modes,
            default_reasoning_mode: model.default_reasoning_mode,
            context_usage_includes_reasoning:
                crate::services::llm::context_usage_includes_reasoning("codex-oauth")
                    .unwrap_or(false),
            interactive_only: false,
        }
    }));
}

async fn add_external_models(
    statuses: &[oauth_providers::OAuthProviderStatus],
    id: ProviderId,
    response: &mut OAuthProviderModelsResponse,
) {
    if !connected(statuses, id) {
        return;
    }
    let provider_id = match id {
        ProviderId::Xai => "xai-oauth",
        ProviderId::Moonshot => "moonshot-oauth",
        ProviderId::OpenAi => return,
    };
    let result = if id == ProviderId::Xai {
        crate::services::llm_oauth::xai_models().await
    } else {
        match crate::services::llm::openai_compat::OpenAiCompatProvider::new(provider_id) {
            Ok(provider) => provider.list_models().await,
            Err(error) => Err(error),
        }
    };
    match result {
        Ok(mut models) => {
            models.retain(|model| crate::services::llm::runtime_models::valid_model_id(&model.id));
            models.truncate(500);
            let canonical_provider = match id {
                ProviderId::Xai => "xai",
                ProviderId::Moonshot => "moonshot",
                ProviderId::OpenAi => return,
            };
            crate::services::llm::runtime_models::replace_provider(canonical_provider, &models);
            response
                .models
                .extend(models.into_iter().map(|model| oauth_model(id, model)));
        }
        Err(error) => response.issues.push(OAuthProviderModelIssue {
            provider_id: id,
            code: catalog_code(&error),
        }),
    }
}

fn connected(statuses: &[oauth_providers::OAuthProviderStatus], provider: ProviderId) -> bool {
    statuses
        .iter()
        .any(|status| status.id == provider && status.connected)
}

fn oauth_model(provider_id: ProviderId, model: ModelInfo) -> OAuthProviderModel {
    let display_name = model
        .display_name
        .clone()
        .unwrap_or_else(|| model.id.clone());
    OAuthProviderModel {
        display_name,
        id: model.id,
        provider_id,
        connection_id: provider_id.usage_connection_id(),
        provider_display_name: provider_id.display_name(),
        context_length: model.context_length,
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_modes: model.reasoning_modes,
        default_reasoning_mode: model.default_reasoning_mode,
        context_usage_includes_reasoning: crate::services::llm::context_usage_includes_reasoning(
            provider_id.usage_connection_id(),
        )
        .unwrap_or(true),
        interactive_only: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_model_transports_public_metadata_without_recalculating_it() {
        let model = oauth_model(
            ProviderId::OpenAi,
            ModelInfo {
                id: "gpt-5.6-sol".to_string(),
                display_name: None,
                owned_by: Some("openai".to_string()),
                context_length: Some(258_400),
                max_output_tokens: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                supports_fast_mode: true,
                reasoning_modes: vec!["high".to_string()],
                default_reasoning_mode: Some("high".to_string()),
                context_usage_includes_reasoning: true,
                is_free: false,
            },
        );

        assert!(model.supports_fast_mode);
        assert_eq!(model.connection_id, "codex-oauth");
        assert_eq!(model.provider_display_name, "OpenAI");
        assert!(!model.context_usage_includes_reasoning);
    }
}
