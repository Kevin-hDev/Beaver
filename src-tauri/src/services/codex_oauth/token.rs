use serde::Deserialize;
use std::time::Duration;
use zeroize::{Zeroize, Zeroizing};

use super::jwt;
use super::store::CodexTokens;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, OAUTH_BODY_LIMIT};

const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

use super::{CLIENT_ID, REDIRECT_URI};

pub async fn exchange_code(code: &str, code_verifier: &str) -> Result<CodexTokens, String> {
    let mut body = format!(
        "grant_type=authorization_code&client_id={CLIENT_ID}&code={}&code_verifier={}&redirect_uri={}",
        urlencoding::encode(code),
        urlencoding::encode(code_verifier),
        urlencoding::encode(REDIRECT_URI),
    );
    let result = post_form(&body).await;
    body.zeroize();
    parse_response(result?).await
}

pub async fn refresh(refresh_val: &str) -> Result<CodexTokens, String> {
    let mut body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={CLIENT_ID}",
        urlencoding::encode(refresh_val),
    );
    let result = post_form(&body).await;
    body.zeroize();
    parse_response(result?).await
}

async fn post_form(body: &str) -> Result<reqwest::Response, String> {
    let client = build_client()?;
    let request = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_string());
    client
        .send_success(request)
        .await
        .map_err(|_| "échange OAuth refusé".to_string())
}

pub async fn ensure_valid() -> Result<CodexTokens, String> {
    let creds = super::store::load()?.ok_or("non connecté à Codex")?;
    if !creds.is_expired() {
        return Ok(creds);
    }
    eprintln!("[codex] session expirée, renouvellement...");
    let new_creds = refresh(&creds.refresh).await?;
    super::store::save(&new_creds)?;
    Ok(new_creds)
}

fn build_client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new(Duration::from_secs(15)).map_err(|_| "erreur interne".to_string())
}

async fn parse_response(resp: reqwest::Response) -> Result<CodexTokens, String> {
    let mut raw: CodexTokenResponse = read_json_bounded(resp, OAUTH_BODY_LIMIT)
        .await
        .map_err(|_| "réponse OAuth invalide".to_string())?;
    if raw.access_token.is_empty() || raw.refresh_token.is_empty() {
        return Err("réponse OAuth invalide".to_string());
    }
    let access = Zeroizing::new(std::mem::take(&mut raw.access_token));
    let refresh_val = Zeroizing::new(std::mem::take(&mut raw.refresh_token));
    let expires_in = raw.expires_in.unwrap_or(3600).clamp(1, 86_400);
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    let claims = jwt::extract_display_claims(&access)?;

    Ok(CodexTokens {
        access,
        refresh: refresh_val,
        expires_at,
        account_hint: Zeroizing::new(claims.account_hint),
    })
}

#[derive(Deserialize)]
struct CodexTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
}

impl Drop for CodexTokenResponse {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
    }
}
