use std::time::Duration;

use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn test_key(provider_id: &str) -> Result<(), String> {
    let key = get_key(provider_id)?;
    test_key_raw(provider_id, key.as_str()).await
}

pub async fn test_key_raw(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    let client = AuthenticatedClient::new(HTTP_TIMEOUT)
        .map_err(|_| "test de la clé impossible".to_string())?;
    let request = if let Some(probe) = llm_probe(provider_id) {
        let probe = probe.map_err(str::to_string)?;
        crate::services::llm::api_key_probe::request(&client, &probe, key)
    } else {
        provider_request(&client, provider_id, key)?
    };
    let response = client
        .send(request)
        .await
        .map_err(|_| "test de la clé impossible".to_string())?;
    check_status(response).await
}

pub async fn test_qwen_key_raw(
    key: &str,
    connection: &crate::services::provider_connections::qwen::QwenConnectionInput,
) -> Result<(), String> {
    reject_unsupported_qwen_key(key)?;
    let endpoint = crate::services::provider_connections::qwen::resolve_qwen_endpoint(connection)
        .map_err(str::to_string)?;
    let client = AuthenticatedClient::new(HTTP_TIMEOUT)
        .map_err(|_| "test de la clé impossible".to_string())?;
    let response = client
        .send(client.get(&endpoint.models_url).bearer_auth(key))
        .await
        .map_err(|_| "test de la clé impossible".to_string())?;
    match qwen_probe_action(response.status().as_u16()) {
        QwenProbeAction::Accept => Ok(()),
        QwenProbeAction::Reject => check_status(response).await,
        QwenProbeAction::ChatFallback => {
            let _ = read_bounded(response, PROVIDER_ERROR_LIMIT).await;
            let url = format!("{}/chat/completions", endpoint.base_url);
            let response = client
                .send(
                    client
                        .post(&url)
                        .bearer_auth(key)
                        .json(&serde_json::json!({
                            "model": "qwen3.8-flash",
                            "max_completion_tokens": 1,
                            "messages": [{"role": "user", "content": "hi"}],
                        })),
                )
                .await
                .map_err(|_| "test de la clé impossible".to_string())?;
            check_status(response).await
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QwenProbeAction {
    Accept,
    ChatFallback,
    Reject,
}

pub(crate) fn qwen_probe_action(status: u16) -> QwenProbeAction {
    match status {
        200..=299 => QwenProbeAction::Accept,
        404 | 405 => QwenProbeAction::ChatFallback,
        _ => QwenProbeAction::Reject,
    }
}

pub(crate) fn reject_unsupported_qwen_key(key: &str) -> Result<(), String> {
    validate::validate_key_value(key)?;
    if key.starts_with("sk-sp-") {
        return Err("provider_configuration_invalid".to_string());
    }
    Ok(())
}

fn llm_probe(
    provider_id: &str,
) -> Option<
    Result<crate::services::llm::api_key_probe::ProbeSpec, &'static str>,
> {
    crate::services::llm::catalog::find_configurable(provider_id)?;
    Some(crate::services::llm::api_key_probe::resolve(provider_id))
}

fn provider_request(
    client: &AuthenticatedClient,
    provider_id: &str,
    key: &str,
) -> Result<reqwest::RequestBuilder, String> {
    let request = match provider_id {
        "google" => client
            .get("https://generativelanguage.googleapis.com/v1beta/models")
            .header("x-goog-api-key", key),
        "brave" => client
            .get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
            .header("X-Subscription-Token", key),
        "exa" => client
            .post("https://api.exa.ai/search")
            .header("x-api-key", key)
            .json(&serde_json::json!({"query":"test","numResults":1})),
        "firecrawl" => client
            .get("https://api.firecrawl.dev/v2/team/credit-usage")
            .bearer_auth(key),
        "nixtla" => client.get("https://api.nixtla.io/models").bearer_auth(key),
        _ => return Err("fournisseur inconnu".to_string()),
    };
    Ok(request)
}

async fn check_status(response: reqwest::Response) -> Result<(), String> {
    let status = response.status().as_u16();
    if (200..=299).contains(&status) {
        return Ok(());
    }
    let _ = read_bounded(response, PROVIDER_ERROR_LIMIT).await;
    match status {
        401 | 403 => Err("Clé API invalide ou non autorisée".into()),
        429 => Err("Clé valide mais quota dépassé".into()),
        _ => Err("test de la clé refusé".into()),
    }
}
