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

use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub use device_flow::DeviceFlowConfig;
pub use headers::request_headers_for;
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

pub async fn login(app: AppHandle, provider: LlmOAuthProvider) -> Result<(), String> {
    let cancel = login_registry::register(provider).await?;
    let expected_generation = store::generation(provider);
    emit_progress(&app, provider, "starting", None, None);
    let result = match provider {
        LlmOAuthProvider::Xai => xai::login(&app, &cancel).await,
        LlmOAuthProvider::Kimi => kimi::login(&app, &cancel).await,
    };
    let outcome = match result {
        Ok(tokens) => {
            let _guard = lifecycle::lock(provider).await;
            if cancel.is_cancelled() {
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
    login_registry::release(provider).await;
    outcome
}

pub async fn cancel(provider: LlmOAuthProvider) {
    login_registry::cancel(provider).await;
}

pub async fn cancel_all() {
    login_registry::cancel_all().await;
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
