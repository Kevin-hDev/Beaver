mod callback;
mod device_flow;
mod headers;
mod kimi;
mod lifecycle;
mod login_registry;
mod oauth_http;
mod refresh;
mod store;
mod types;
mod xai;
mod xai_catalog;
#[cfg(test)]
mod xai_catalog_tests;
mod xai_catalog_wire;
mod xai_headers;
mod xai_identity;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::services::work_registry::ServiceWorkCancellation;

pub use device_flow::DeviceFlowConfig;
pub use headers::request_headers_with_identity;
pub(crate) use xai_catalog_wire::{XaiBackend, XaiCatalogModel};
pub(crate) use xai_headers::model_header as xai_model_header;
pub use xai_headers::PROXY_BASE_URL as XAI_PROXY_BASE_URL;

pub async fn xai_models(
) -> Result<Vec<crate::services::llm::types::ModelInfo>, crate::services::llm::types::LlmError> {
    xai_catalog::list_models().await
}

pub(crate) async fn xai_catalog_model(model: &str) -> Result<XaiCatalogModel, String> {
    xai_catalog::model(model)
        .await
        .map_err(|_| "provider_configuration_invalid".to_string())
}
pub use types::{AccessToken, DeviceAuthorization, LlmOAuthProvider, OAuthFailure, TokenBundle};

const PROGRESS_EVENT: &str = "oauth-login-progress";

#[derive(Clone, Serialize)]
struct LoginProgress<'a> {
    provider_id: &'a str,
    stage: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_code: Option<&'a str>,
}

pub async fn login(
    app: AppHandle,
    provider: LlmOAuthProvider,
    work_cancel: ServiceWorkCancellation,
) -> Result<(), String> {
    let registered = login_registry::register(provider).await?;
    let expected_generation = store::generation(provider);
    emit_progress(&app, provider, "starting", None, None);
    let provider_login = async {
        match provider {
            LlmOAuthProvider::Xai => xai::login(&app, &registered.cancel).await,
            LlmOAuthProvider::Kimi => kimi::login(&app, &registered.cancel).await,
        }
    };
    let result = tokio::select! {
        result = provider_login => result,
        _ = work_cancel.cancelled() => Err(OAuthFailure::Cancelled),
    };
    let outcome = match result {
        Ok(mut tokens) => {
            if provider == LlmOAuthProvider::Xai
                && xai_identity::enrich(&mut tokens, None).await.is_err()
            {
                emit_progress(&app, provider, "error", None, None);
                registered.completion.complete(());
                return Err("Connexion impossible".to_string());
            }
            let _guard = lifecycle::lock(provider).await;
            if registered.cancel.is_cancelled() {
                emit_progress(&app, provider, "cancelled", None, None);
                Err("Connexion annulée".to_string())
            } else {
                match store::save_if_generation(provider, &tokens, expected_generation) {
                    Ok(_) => {
                        emit_progress(&app, provider, "success", None, None);
                        Ok(())
                    }
                    Err(_) => {
                        emit_progress(&app, provider, "error", None, None);
                        Err("Connexion impossible".to_string())
                    }
                }
            }
        }
        Err(OAuthFailure::Cancelled) => {
            emit_progress(&app, provider, "cancelled", None, None);
            Err("Connexion annulée".to_string())
        }
        Err(_) => {
            emit_progress(&app, provider, "error", None, None);
            Err("Connexion impossible".to_string())
        }
    };
    registered.completion.complete(());
    outcome
}

pub async fn cancel(provider: LlmOAuthProvider) {
    login_registry::cancel(provider).await;
}

pub async fn logout(provider: LlmOAuthProvider) -> Result<(), String> {
    login_registry::cancel(provider).await;
    let _guard = lifecycle::lock(provider).await;
    store::clear(provider)
}

pub async fn invalidate(provider: LlmOAuthProvider) {
    let _guard = lifecycle::lock(provider).await;
    let _ = store::clear(provider);
}

pub fn is_connected(provider: LlmOAuthProvider) -> bool {
    store::load(provider).ok().flatten().is_some()
}

pub async fn access_token(provider: LlmOAuthProvider) -> Result<AccessToken, String> {
    refresh::access_token(provider).await
}

pub async fn force_refresh(
    provider: LlmOAuthProvider,
    generation: u64,
) -> Result<AccessToken, String> {
    refresh::force_refresh(provider, generation).await
}

pub(crate) fn emit_progress<'a>(
    app: &AppHandle,
    provider: LlmOAuthProvider,
    stage: &'a str,
    verification_url: Option<&'a str>,
    user_code: Option<&'a str>,
) {
    let _ = app.emit(
        PROGRESS_EVENT,
        LoginProgress {
            provider_id: provider.provider_id().trim_end_matches("-oauth"),
            stage,
            hint: None,
            verification_url,
            user_code,
        },
    );
}
