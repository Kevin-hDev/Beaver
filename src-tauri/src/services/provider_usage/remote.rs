use super::types::RemoteData;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient};
use std::time::{Duration, Instant};

const RESPONSE_LIMIT: usize = 256 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

pub async fn resolve(connection_id: &str, force_refresh: bool) -> RemoteData {
    let requested_at = Instant::now();
    if !force_refresh {
        if let Some(cached) = super::cache::get(connection_id).await {
            return cached;
        }
    }
    let Some(mut gate) = super::remote_gate::lock(connection_id).await else {
        return local_only(connection_id);
    };
    if super::remote_gate::should_skip(&gate, requested_at, force_refresh) {
        return cached_or_fallback(connection_id).await;
    }
    if !force_refresh {
        if let Some(cached) = super::cache::get(connection_id).await {
            return cached;
        }
    }
    let Some(generation) = super::credential_epoch::current(connection_id) else {
        super::remote_gate::complete(&mut gate);
        return local_only(connection_id);
    };
    let result = match fetch(connection_id).await {
        Ok(remote) => {
            if !super::credential_epoch::is_current(connection_id, generation) {
                local_only(connection_id)
            } else {
                super::cache::put(connection_id, generation, remote.clone()).await;
                let _ = super::ledger::save_remote(connection_id, generation, remote.clone()).await;
                remote
            }
        }
        Err(()) => fallback(connection_id).await,
    };
    super::remote_gate::complete(&mut gate);
    result
}

async fn cached_or_fallback(connection_id: &str) -> RemoteData {
    match super::cache::get(connection_id).await {
        Some(cached) => cached,
        None => fallback(connection_id).await,
    }
}

async fn fetch(connection_id: &str) -> Result<RemoteData, ()> {
    match connection_id {
        "openrouter" => fetch_api(connection_id, "https://openrouter.ai/api/v1/key").await,
        "deepseek" => fetch_api(connection_id, "https://api.deepseek.com/user/balance").await,
        "moonshot" => fetch_api(connection_id, "https://api.moonshot.ai/v1/users/me/balance").await,
        "codex-oauth" => fetch_codex().await,
        "moonshot-oauth" => fetch_kimi().await,
        "groq" | "cerebras" => Ok(recent_headers(connection_id).await),
        _ => Ok(local_only(connection_id)),
    }
}

async fn recent_headers(connection_id: &str) -> RemoteData {
    let generation = super::credential_epoch::current(connection_id).unwrap_or_default();
    super::ledger::recent_remote(connection_id, generation)
        .await
        .unwrap_or_else(|| local_only(connection_id))
}

async fn fetch_api(connection_id: &str, url: &str) -> Result<RemoteData, ()> {
    let key = crate::services::api_keys::get_key(connection_id).map_err(|_| ())?;
    let client = client()?;
    let response = client
        .send_success(client.get(url).bearer_auth(key.as_str()))
        .await
        .map_err(|_| ())?;
    let body: serde_json::Value = read_json_bounded(response, RESPONSE_LIMIT)
        .await
        .map_err(|_| ())?;
    super::remote_api::parse(connection_id, &body).ok_or(())
}

async fn fetch_codex() -> Result<RemoteData, ()> {
    let credentials = crate::services::codex_oauth::token::ensure_valid()
        .await
        .map_err(|_| ())?;
    let client = client()?;
    let response = client
        .send_success(
            client
                .get(CODEX_USAGE_URL)
                .bearer_auth(credentials.access.as_str())
                .header("chatgpt-account-id", credentials.account_hint.as_str()),
        )
        .await
        .map_err(|_| ())?;
    parse_oauth("codex-oauth", response).await
}

async fn fetch_kimi() -> Result<RemoteData, ()> {
    let route = crate::services::llm::route::resolve("moonshot-oauth").ok_or(())?;
    let client = client()?;
    let url = format!("{}/usages", route.base_url);
    let response = route
        .send_authenticated(
            &client,
            crate::services::llm::request_purpose::RequestPurpose::AccountMetadata,
            |token, headers| client.get(&url).headers(headers).bearer_auth(token),
        )
        .await
        .map_err(|_| ())?;
    if !response.status().is_success() {
        return Err(());
    }
    parse_oauth("moonshot-oauth", response).await
}

async fn parse_oauth(connection_id: &str, response: reqwest::Response) -> Result<RemoteData, ()> {
    let body: serde_json::Value = read_json_bounded(response, RESPONSE_LIMIT)
        .await
        .map_err(|_| ())?;
    super::remote_oauth::parse(connection_id, &body).ok_or(())
}

async fn fallback(connection_id: &str) -> RemoteData {
    let generation = super::credential_epoch::current(connection_id).unwrap_or_default();
    if let Some(mut previous) = super::ledger::recent_remote(connection_id, generation).await {
        previous.notice_code = Some("usage_fetch_failed".into());
        return previous;
    }
    let mut remote = local_only(connection_id);
    remote.notice_code = Some("usage_fetch_failed".into());
    remote
}

fn local_only(connection_id: &str) -> RemoteData {
    RemoteData {
        notice_code: Some(if connection_id == "xai-oauth" {
            "provider_usage_site_only".into()
        } else {
            "provider_account_usage_unavailable".into()
        }),
        fetched_at: chrono::Utc::now().timestamp(),
        ..Default::default()
    }
}

fn client() -> Result<AuthenticatedClient, ()> {
    AuthenticatedClient::new(REQUEST_TIMEOUT).map_err(|_| ())
}
