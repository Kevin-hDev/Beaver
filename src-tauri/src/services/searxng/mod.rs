pub mod lifecycle;
#[cfg(test)]
mod lifecycle_tests;

mod client;
mod compat;
mod paths;
mod process;
mod runtime;
mod settings;
mod source_filter;
mod wheels;
mod work_supervision;

pub use lifecycle::SearxngSidecar;

use crate::services::agent_local::types_tools::SearchResult;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    lifecycle::search(query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    lifecycle::prepare_on_startup(app);
}
