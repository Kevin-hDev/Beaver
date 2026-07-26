use rand::RngCore;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::{callback, jwt, pkce, store, token};

use super::{CLIENT_ID, REDIRECT_URI};

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

fn build_auth_url(challenge: &str, state: &str) -> String {
    format!(
        "{AUTH_URL}?response_type=code\
         &client_id={CLIENT_ID}\
         &redirect_uri={}\
         &scope={}\
         &code_challenge={challenge}\
         &code_challenge_method=S256\
         &state={state}",
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPES),
    )
}

pub async fn login() -> Result<String, String> {
    let cancel = register_login().await?;
    let result = login_registered(&cancel).await;
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
    let url = build_auth_url(&pair.challenge, &state);

    open::that(&url).map_err(|_| "impossible d'ouvrir le navigateur".to_string())?;
    eprintln!("[codex] navigateur ouvert, attente du callback...");

    let cb = callback::wait_for_callback(&state, cancel).await?;
    eprintln!("[codex] code reçu, échange en cours...");

    let creds = token::exchange_code(cb.code.as_str(), pair.verifier.as_str()).await?;
    let email = jwt::extract_display_claims(&creds.access)
        .ok()
        .and_then(|c| c.email)
        .unwrap_or_else(|| "inconnu".to_string());

    store::save(&creds)?;
    eprintln!("[codex] connecté");

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

pub fn logout() -> Result<(), String> {
    store::clear()
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
}
