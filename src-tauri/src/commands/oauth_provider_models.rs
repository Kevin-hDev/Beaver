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
    pub display_name: String,
    pub context_length: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub reasoning_modes: Vec<String>,
    pub default_reasoning_mode: Option<String>,
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
    response.models.extend(models.into_iter().map(|model| {
        let display_name = model
            .display_name
            .clone()
            .unwrap_or_else(|| model.id.clone());
        OAuthProviderModel {
            display_name,
            id: model.id,
            provider_id: ProviderId::OpenAi,
            context_length: model.context_length,
            supports_tools: model.supports_tools,
            supports_vision: model.supports_vision,
            supports_thinking: model.supports_thinking,
            reasoning_modes: model.reasoning_modes,
            default_reasoning_mode: model.default_reasoning_mode,
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
    let result = crate::services::llm::openai_compat::OpenAiCompatProvider::new(provider_id);
    let result = match result {
        Ok(provider) => provider.list_models().await,
        Err(error) => Err(error),
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
        context_length: model.context_length,
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        reasoning_modes: model.reasoning_modes,
        default_reasoning_mode: model.default_reasoning_mode,
        interactive_only: true,
    }
}
