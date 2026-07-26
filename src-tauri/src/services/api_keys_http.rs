use std::time::Duration;

use crate::services::secure_http::{
    read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT,
};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn test_key(provider_id: &str) -> Result<(), String> {
    let key = get_key(provider_id)?;
    test_key_raw(provider_id, key.as_str()).await
}

pub async fn test_key_raw(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    let client = AuthenticatedClient::new(HTTP_TIMEOUT)
        .map_err(|_| "test de la clé impossible".to_string())?;
    let request = if let Some(spec) = crate::services::llm::catalog::find(provider_id) {
        if !spec.models_endpoint.is_empty() {
            client
                .get(format!("{}{}", spec.base_url, spec.models_endpoint))
                .bearer_auth(key)
        } else {
            let model = crate::services::llm::openai_compat::ping_model(provider_id);
            client
                .post(format!("{}/chat/completions", spec.base_url))
                .bearer_auth(key)
                .json(&serde_json::json!({
                    "model": model,
                    "max_tokens": 1,
                    "messages": [{"role":"user","content":"hi"}]
                }))
        }
    } else {
        provider_request(&client, provider_id, key)?
    };
    let response = client
        .send(request)
        .await
        .map_err(|_| "test de la clé impossible".to_string())?;
    check_status(response).await
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
        "nixtla" => client
            .get("https://api.nixtla.io/models")
            .bearer_auth(key),
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
