use crate::services::agent_local::types_tools::SearchResult;
use crate::services::search::common;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(15);

pub async fn search(base_url: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    let query = common::validate_query(query)?;
    let url = format!("{}/search", base_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .get(url)
        .query(&[("q", query.as_str()), ("format", "json")])
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("SearXNG: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("SearXNG: HTTP {}", resp.status()));
    }

    let json = common::read_json_bounded(resp, "SearXNG").await?;
    Ok(parse_results(&json))
}

fn parse_results(json: &serde_json::Value) -> Vec<SearchResult> {
    json["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            common::make_result(
                r["title"].as_str().unwrap_or(""),
                r["url"].as_str().unwrap_or(""),
                r["content"]
                    .as_str()
                    .or_else(|| r["snippet"].as_str())
                    .unwrap_or(""),
            )
        })
        .take(common::MAX_RESULTS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_results_bounds_and_filters() {
        let parsed = parse_results(&json!({
            "results": [
                {"title": "ok", "url": "https://example.com", "content": "body"},
                {"title": "bad", "url": "file:///tmp/a", "content": "ignored"}
            ]
        }));
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].url, "https://example.com");
    }
}
