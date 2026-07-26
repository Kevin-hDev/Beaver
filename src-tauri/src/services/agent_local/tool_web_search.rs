use crate::services::agent_local::types_tools::SearchResult;
use crate::services::search;

pub async fn web_search(query: &str) -> Result<Vec<SearchResult>, String> {
    search::run_search(query).await
}
