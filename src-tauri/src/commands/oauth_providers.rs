use crate::services::oauth_providers::{self, ProviderId};
use tauri::Emitter;

const STATUS_EVENT: &str = "oauth-provider-status-changed";
const PROGRESS_EVENT: &str = "oauth-login-progress";
#[tauri::command]
pub async fn list_oauth_provider_statuses() -> Vec<oauth_providers::OAuthProviderStatus> {
    oauth_providers::list_statuses()
}

#[tauri::command]
pub async fn start_oauth_provider_login(
    app: tauri::AppHandle,
    work: tauri::State<'_, crate::services::oauth_work::OAuthWorkServices>,
    provider_id: String,
    diagnostic_id: String,
) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    match provider {
        ProviderId::OpenAi => {
            emit_openai_progress(&app, "waiting");
            if crate::commands::codex_login_with_work(app.clone(), &work)
                .await
                .is_err()
            {
                emit_openai_progress(&app, "error");
                return Err("Connexion impossible".to_string());
            }
            emit_openai_progress(&app, "success");
        }
        ProviderId::Moonshot | ProviderId::Xai => {
            let login_app = app.clone();
            work.run(move |cancel| {
                oauth_providers::login_external(login_app, provider, diagnostic_id, cancel)
            })
            .await
            .map_err(|_| "Connexion impossible".to_string())??;
        }
    }
    crate::services::provider_usage::invalidate_remote(provider.usage_connection_id()).await;
    let _ = app.emit(STATUS_EVENT, ());
    Ok(())
}

#[tauri::command]
pub async fn cancel_oauth_provider_login(provider_id: String) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    if provider == ProviderId::OpenAi {
        crate::services::codex_oauth::login::cancel_login().await;
    } else {
        oauth_providers::cancel_login(provider).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn disconnect_oauth_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    match provider {
        ProviderId::OpenAi => crate::commands::codex_logout(app.clone()).await?,
        ProviderId::Moonshot | ProviderId::Xai => {
            oauth_providers::logout_external(provider).await?;
        }
    }
    crate::services::provider_usage::invalidate_remote(provider.usage_connection_id()).await;
    let _ = app.emit(STATUS_EVENT, ());
    Ok(())
}

fn emit_openai_progress(app: &tauri::AppHandle, stage: &'static str) {
    let _ = app.emit(
        PROGRESS_EVENT,
        oauth_providers::OAuthLoginProgress {
            provider_id: ProviderId::OpenAi,
            stage,
            hint: None,
            verification_url: None,
            user_code: None,
        },
    );
}
