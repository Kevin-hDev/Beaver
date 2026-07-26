use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::{callback, device_flow, emit_progress, oauth_http, OAuthFailure, TokenBundle};
use super::{DeviceFlowConfig, LlmOAuthProvider};

pub const CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
pub const TOKEN_URL: &str = "https://auth.x.ai/oauth2/token";
const AUTH_URL: &str = "https://auth.x.ai/oauth2/authorize";
const DEVICE_URL: &str = "https://auth.x.ai/oauth2/device/code";
const REDIRECT_URI: &str = "http://127.0.0.1:56121/callback";
const SCOPES: &str = "openid profile email offline_access grok-cli:access api:access";

pub async fn login(
    app: &AppHandle,
    cancel: &CancellationToken,
) -> Result<TokenBundle, OAuthFailure> {
    let (verifier, challenge) = crate::services::mcp_oauth::pkce::generate();
    let state = generate_state();
    let nonce = generate_state();
    let receiver = match callback::start(state.clone(), cancel.clone()).await {
        Ok(receiver) => receiver,
        Err(_) => return login_device(app, cancel).await,
    };
    let url = build_auth_url(&challenge, &state, &nonce)?;
    emit_progress(app, LlmOAuthProvider::Xai, "browser_open", None, None);
    let _ = open::that(&url);
    let code = receiver.await.map_err(|_| OAuthFailure::Generic)??;
    let response = oauth_http::post_form(
        LlmOAuthProvider::Xai,
        TOKEN_URL,
        &[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", code.as_str()),
            ("redirect_uri", REDIRECT_URI),
            ("code_verifier", verifier.as_str()),
        ],
    )
    .await?;
    oauth_http::parse_token(response, None).await
}

pub async fn refresh(refresh_token: &str) -> Result<TokenBundle, OAuthFailure> {
    oauth_http::refresh(LlmOAuthProvider::Xai, TOKEN_URL, CLIENT_ID, refresh_token).await
}

async fn login_device(
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
        LlmOAuthProvider::Xai,
        "device_code",
        Some(&public_url),
        Some(&authorization.user_code),
    );
    let _ = open::that(url);
    device_flow::poll(&config, &authorization, cancel).await
}

fn device_config() -> DeviceFlowConfig {
    DeviceFlowConfig {
        provider: LlmOAuthProvider::Xai,
        client_id: CLIENT_ID,
        device_url: DEVICE_URL,
        token_url: TOKEN_URL,
        scope: Some(SCOPES),
    }
}

fn generate_state() -> Zeroizing<String> {
    let mut bytes = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let value = Zeroizing::new(URL_SAFE_NO_PAD.encode(bytes));
    bytes.fill(0);
    value
}

fn build_auth_url(challenge: &str, state: &str, nonce: &str) -> Result<String, OAuthFailure> {
    let mut url = url::Url::parse(AUTH_URL).map_err(|_| OAuthFailure::Generic)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("scope", SCOPES)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state)
        .append_pair("nonce", nonce)
        .append_pair("plan", "generic")
        .append_pair("referrer", "cl-go-dash");
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_url_uses_pkce_and_cl_go_referrer() {
        let state = generate_state();
        let nonce = generate_state();
        let url = build_auth_url("challenge", &state, &nonce).unwrap();
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("referrer=cl-go-dash"));
        assert!(url.contains("127.0.0.1%3A56121"));
        assert!(url.contains("nonce="));
    }

    #[test]
    fn pkce_challenge_matches_the_zeroized_verifier() {
        use sha2::{Digest, Sha256};
        let (verifier, challenge) = crate::services::mcp_oauth::pkce::generate();
        assert_eq!(
            challenge,
            URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
        );
        assert!((43..=128).contains(&verifier.len()));
    }
}
