use crate::services::agent_local::types_tools::SearchResult;
use crate::services::search;

pub async fn web_search(query: &str) -> Result<Vec<SearchResult>, search::SearchFailure> {
    search::run_search(query).await
}
