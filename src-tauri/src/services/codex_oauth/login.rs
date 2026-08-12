use rand::RngCore;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::CLIENT_ID;
use super::{callback_server::CallbackServer, jwt, pkce, token};
use crate::services::work_registry::ServiceWorkCancellation;

const AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const SCOPES: &str = "openid profile email offline_access";
static ACTIVE_LOGIN: LazyLock<Mutex<Option<CancellationToken>>> =
    LazyLock::new(|| Mutex::new(None));

fn generate_state() -> Zeroizing<String> {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let s = hex::encode(bytes);
    bytes.fill(0);
    Zeroizing::new(s)
}

fn build_auth_url(
    challenge: &str,
    state: &str,
    redirect_uri: &str,
) -> Result<Zeroizing<String>, String> {
    if challenge.len() != 43
        || !challenge
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("configuration OAuth invalide".to_string());
    }
    super::callback::validate_state(state)?;
    validate_redirect_uri(redirect_uri)?;
    Ok(Zeroizing::new(format!(
        "{AUTH_URL}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&id_token_add_organizations=true&codex_cli_simplified_flow=true&state={}&originator={}",
        urlencoding::encode(CLIENT_ID),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(SCOPES),
        urlencoding::encode(challenge),
        urlencoding::encode(state),
        urlencoding::encode(super::ORIGINATOR),
    )))
}

fn validate_redirect_uri(redirect_uri: &str) -> Result<(), String> {
    let parsed = url::Url::parse(redirect_uri).map_err(|_| "configuration OAuth invalide")?;
    let valid = parsed.scheme() == "http"
        && parsed.host_str() == Some("localhost")
        && matches!(parsed.port(), Some(1455 | 1457))
        && parsed.path() == "/auth/callback"
        && parsed.query().is_none()
        && parsed.fragment().is_none()
        && parsed.username().is_empty()
        && parsed.password().is_none();
    if valid {
        Ok(())
    } else {
        Err("configuration OAuth invalide".to_string())
    }
}

pub async fn login(work_cancel: ServiceWorkCancellation) -> Result<String, String> {
    let cancel = register_login().await?;
    let result = tokio::select! {
        result = login_registered(&cancel) => result,
        _ = work_cancel.cancelled() => Err("Connexion annulée".to_string()),
    };
    *ACTIVE_LOGIN.lock().await = None;
    result
}

async fn register_login() -> Result<CancellationToken, String> {
    let mut active = ACTIVE_LOGIN.lock().await;
    if active.is_some() {
        return Err("Connexion déjà en cours".to_string());
    }
    let cancel = CancellationToken::new();
    *active = Some(cancel.clone());
    Ok(cancel)
}

async fn login_registered(cancel: &CancellationToken) -> Result<String, String> {
    let pair = pkce::generate();
    let state = generate_state();
    let server = CallbackServer::bind().await?;
    let redirect_uri = server.redirect_uri();
    let url = build_auth_url(&pair.challenge, &state, &redirect_uri)?;

    open::that(url.as_str()).map_err(|_| "impossible d'ouvrir le navigateur".to_string())?;
    let cb = server.wait(&state, cancel).await?;

    let creds =
        token::exchange_code(cb.code.as_str(), pair.verifier.as_str(), &redirect_uri).await?;
    let email = jwt::extract_display_claims(&creds.access)
        .ok()
        .and_then(|c| c.email)
        .unwrap_or_else(|| "inconnu".to_string());

    token::save_login(&creds).await?;
    Ok(email)
}

pub async fn cancel_login() {
    let token = { ACTIVE_LOGIN.lock().await.as_ref().cloned() };
    if let Some(token) = token {
        token.cancel();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if ACTIVE_LOGIN.lock().await.is_none() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;
    }
}

pub async fn logout() -> Result<(), String> {
    token::clear_login().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_waits_until_the_callback_slot_is_released() {
        let token = register_login().await.expect("login slot");
        let cleanup = tokio::spawn(async move {
            token.cancelled().await;
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            *ACTIVE_LOGIN.lock().await = None;
        });

        let started = std::time::Instant::now();
        cancel_login().await;

        assert!(ACTIVE_LOGIN.lock().await.is_none());
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
        cleanup.await.expect("cleanup task");
    }

    #[test]
    fn authorization_url_uses_current_codex_parameters() {
        let challenge = "a".repeat(43);
        let state = "0123456789abcdef0123456789abcdef";
        let url = build_auth_url(&challenge, state, "http://localhost:1457/auth/callback")
            .expect("authorization URL");
        let parsed = url::Url::parse(url.as_str()).unwrap();
        let query: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some("http://localhost:1457/auth/callback")
        );
        assert_eq!(
            query.get("id_token_add_organizations").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            query.get("codex_cli_simplified_flow").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            query.get("originator").map(String::as_str),
            Some(crate::services::codex_oauth::ORIGINATOR)
        );
    }

    #[test]
    fn authorization_url_rejects_unregistered_callback_targets() {
        let challenge = "a".repeat(43);
        let state = "0123456789abcdef0123456789abcdef";

        assert!(build_auth_url(&challenge, state, "http://localhost:9999/auth/callback").is_err());
        assert!(build_auth_url(&challenge, state, "https://example.com/auth/callback").is_err());
    }
}
