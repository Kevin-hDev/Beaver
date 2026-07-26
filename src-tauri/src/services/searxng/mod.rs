pub mod lifecycle;

mod client;
mod compat;
mod paths;
mod process;
mod runtime;
mod settings;
mod source_filter;
mod wheels;

pub use lifecycle::{is_ready, SearxngSidecar};

use crate::services::agent_local::types_tools::SearchResult;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    lifecycle::search(query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    lifecycle::prepare_on_startup(app);
}

pub async fn stop(sidecar: &SearxngSidecar) {
    lifecycle::stop(sidecar).await;
}
