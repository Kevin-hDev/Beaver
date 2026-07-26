//! Client Firecrawl — extraction Markdown LLM-ready d'une URL ou recherche.
//!
//! Endpoints :
//! - POST https://api.firecrawl.dev/v2/search : recherche + extraction
//! - POST https://api.firecrawl.dev/v2/scrape : extraction d'une URL unique
//!
//! Auth : `Authorization: Bearer {key}`

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use crate::services::search::common;
use crate::services::secure_http::AuthenticatedClient;
use std::time::Duration;

const SEARCH_URL: &str = "https://api.firecrawl.dev/v2/search";
const CREDIT_URL: &str = "https://api.firecrawl.dev/v2/team/credit-usage";
const TIMEOUT: Duration = Duration::from_secs(30);

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let query = common::validate_query(query)?;
    let key = api_keys::get_key("firecrawl")?;
    let client = AuthenticatedClient::new(TIMEOUT).map_err(|_| "Firecrawl: erreur interne")?;

    let payload = serde_json::json!({
        "query": query,
        "limit": common::MAX_RESULTS,
    });

    let request = client.post(SEARCH_URL).bearer_auth(&*key).json(&payload);
    let resp = client
        .send(request)
        .await
        .map_err(|_| "Firecrawl: requête impossible".to_string())?;
    let resp = common::ensure_success(resp, "Firecrawl").await?;

    let json = common::read_json_bounded(resp, "Firecrawl").await?;

    let results = json["data"]["web"]
        .as_array()
        .or_else(|| json["data"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|r| {
            common::make_result(
                r["title"].as_str().unwrap_or(""),
                r["url"].as_str().unwrap_or(""),
                r["description"]
                    .as_str()
                    .unwrap_or_else(|| r["markdown"].as_str().unwrap_or("")),
            )
        })
        .take(common::MAX_RESULTS)
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let key = api_keys::get_key("firecrawl")?;
    let client = AuthenticatedClient::new(Duration::from_secs(10))
        .map_err(|_| "Firecrawl: erreur interne")?;

    let request = client.get(CREDIT_URL).bearer_auth(&*key);
    let resp = client
        .send(request)
        .await
        .map_err(|_| "Firecrawl: requête impossible".to_string())?;

    if resp.status().is_success() {
        Ok(())
    } else if resp.status().as_u16() == 401 {
        Err("Clé Firecrawl invalide".into())
    } else {
        Err("Firecrawl: requête refusée".into())
    }
}
