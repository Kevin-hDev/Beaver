//! Client Brave Search API avec rate limiter (1 req/s, free tier).

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use crate::services::search::common;
use crate::services::secure_http::AuthenticatedClient;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const URL: &str = "https://api.search.brave.com/res/v1/web/search";
const TIMEOUT: Duration = Duration::from_secs(10);
const MIN_INTERVAL: Duration = Duration::from_millis(1100);

static NEXT_ALLOWED: Mutex<Option<Instant>> = Mutex::new(None);

async fn wait_rate_limit() {
    let wait = {
        let mut next = NEXT_ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        match *next {
            Some(t) if t > now => {
                let delay = t - now;
                *next = Some(t + MIN_INTERVAL);
                Some(delay)
            }
            _ => {
                *next = Some(now + MIN_INTERVAL);
                None
            }
        }
    };
    if let Some(d) = wait {
        tokio::time::sleep(d).await;
    }
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let query = common::validate_query(query)?;
    wait_rate_limit().await;
    let key = api_keys::get_key("brave")?;
    let client = AuthenticatedClient::new(TIMEOUT).map_err(|_| "Brave: erreur interne")?;
    let request = client
        .get(URL)
        .query(&[
            ("q", query.as_str()),
            ("count", &common::MAX_RESULTS.to_string()),
        ])
        .header("X-Subscription-Token", &*key);
    let resp = client
        .send(request)
        .await
        .map_err(|_| "Brave: requête impossible".to_string())?;
    let resp = common::ensure_success(resp, "Brave").await?;

    let json = common::read_json_bounded(resp, "Brave").await?;

    let results = json["web"]["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            common::make_result(
                r["title"].as_str().unwrap_or(""),
                r["url"].as_str().unwrap_or(""),
                r["description"].as_str().unwrap_or(""),
            )
        })
        .take(common::MAX_RESULTS)
        .collect();
    Ok(results)
}

pub async fn test_connection() -> Result<(), String> {
    let _ = search("test").await?;
    Ok(())
}
