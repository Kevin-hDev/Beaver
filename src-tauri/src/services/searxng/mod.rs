pub mod lifecycle;
#[cfg(test)]
mod lifecycle_tests;

mod client;
mod compat;
mod paths;
mod process;
mod python_runtime;
mod python_runtime_path;
mod runtime;
mod runtime_error;
mod runtime_manifest;
mod settings;
mod source_filter;
mod start_lifecycle;
mod startup;
mod startup_failure;
mod stop_lifecycle;
mod wheels;
mod work_supervision;

#[cfg(test)]
mod python_runtime_tests;
#[cfg(test)]
mod runtime_error_tests;
#[cfg(test)]
mod runtime_manifest_tests;

pub use lifecycle::SearxngSidecar;

use crate::services::agent_local::types_tools::SearchResult;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    lifecycle::search(query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    lifecycle::prepare_on_startup(app);
}
