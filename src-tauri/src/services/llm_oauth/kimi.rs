use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{device_flow, emit_progress, oauth_http, DeviceFlowConfig};
use super::{LlmOAuthProvider, OAuthFailure, TokenBundle};

pub const CLIENT_ID: &str = "17e5f671-d194-4dfb-9706-5516cb48c098";
pub const TOKEN_URL: &str = "https://auth.kimi.com/api/oauth/token";
const DEVICE_URL: &str = "https://auth.kimi.com/api/oauth/device_authorization";

pub async fn login(
    app: &AppHandle,
    cancel: &CancellationToken,
) -> Result<TokenBundle, OAuthFailure> {
    let config = device_config();
    let authorization = device_flow::request(&config).await?;
    let url = authorization
        .verification_uri_complete
        .as_deref()
        .unwrap_or(&authorization.verification_uri);
    let public_url = device_flow::public_verification_url(&authorization)?;
    emit_progress(
        app,
        LlmOAuthProvider::Kimi,
        "device_code",
        Some(&public_url),
        Some(&authorization.user_code),
    );
    let _ = open::that(url);
    device_flow::poll(&config, &authorization, cancel).await
}

pub async fn refresh(refresh_token: &str) -> Result<TokenBundle, OAuthFailure> {
    oauth_http::refresh(LlmOAuthProvider::Kimi, TOKEN_URL, CLIENT_ID, refresh_token).await
}

fn device_config() -> DeviceFlowConfig {
    DeviceFlowConfig {
        provider: LlmOAuthProvider::Kimi,
        client_id: CLIENT_ID,
        device_url: DEVICE_URL,
        token_url: TOKEN_URL,
        scope: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_official_rfc8628_endpoints() {
        let config = device_config();
        assert_eq!(
            config.device_url,
            "https://auth.kimi.com/api/oauth/device_authorization"
        );
        assert_eq!(config.token_url, "https://auth.kimi.com/api/oauth/token");
        assert_eq!(config.client_id, CLIENT_ID);
    }
}
