mod types;

use crate::services::llm_oauth::{self, LlmOAuthProvider};

pub use types::{OAuthLoginProgress, OAuthProviderStatus, ProviderId};

pub fn list_statuses() -> Vec<OAuthProviderStatus> {
    let codex = crate::services::codex_oauth::store::load().ok().flatten();
    let codex_account = codex.as_ref().and_then(|tokens| {
        crate::services::codex_oauth::jwt::extract_display_claims(&tokens.access)
            .ok()
            .and_then(|claims| claims.email)
    });
    vec![
        OAuthProviderStatus {
            id: ProviderId::OpenAi,
            display_name: "OpenAI",
            connected: codex.is_some(),
            account: codex_account,
            experimental: false,
        },
        external_status(ProviderId::Xai),
        external_status(ProviderId::Moonshot),
    ]
}

pub async fn login_external(
    app: tauri::AppHandle,
    provider: ProviderId,
    _diagnostic_id: &str,
) -> Result<(), String> {
    llm_oauth::login(app, provider.as_llm_oauth()?).await
}

pub async fn cancel_login(provider: ProviderId) {
    if let Ok(provider) = provider.as_llm_oauth() {
        llm_oauth::cancel(provider).await;
    }
}

pub async fn cancel_all() {
    llm_oauth::cancel_all().await;
}

pub async fn logout_external(provider: ProviderId) -> Result<(), String> {
    llm_oauth::logout(provider.as_llm_oauth()?).await
}

fn external_status(id: ProviderId) -> OAuthProviderStatus {
    let connected = id.as_llm_oauth().is_ok_and(llm_oauth::is_connected);
    OAuthProviderStatus {
        id,
        display_name: id.display_name(),
        connected,
        account: None,
        experimental: id == ProviderId::Moonshot,
    }
}

impl ProviderId {
    fn as_llm_oauth(self) -> Result<LlmOAuthProvider, String> {
        match self {
            Self::Xai => Ok(LlmOAuthProvider::Xai),
            Self::Moonshot => Ok(LlmOAuthProvider::Kimi),
            Self::OpenAi => Err("Fournisseur invalide".to_string()),
        }
    }
}
