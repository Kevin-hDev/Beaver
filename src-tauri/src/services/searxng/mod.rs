pub mod lifecycle;
#[cfg(test)]
mod lifecycle_tests;

mod client;
mod compat;
pub(crate) mod error_codes;
mod generational_publication;
mod paths;
mod private_file;
mod process;
mod process_receipt;
mod python_runtime;
mod python_runtime_path;
mod runtime;
mod runtime_command;
mod runtime_command_drain;
mod runtime_command_finish;
mod runtime_command_log;
mod runtime_environment;
mod runtime_environment_fs;
mod runtime_error;
mod runtime_manifest;
mod runtime_receipt;
mod settings;
mod source_filter;
mod start_lifecycle;
mod start_process_receipt;
mod start_readiness;
mod startup;
mod startup_failure;
mod stop_lifecycle;
mod wheels;
mod work_supervision;

#[cfg(test)]
mod process_receipt_tests;
#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod python_runtime_tests;
#[cfg(test)]
mod runtime_command_tests;
#[cfg(test)]
mod runtime_environment_tests;
#[cfg(test)]
mod runtime_error_tests;
#[cfg(test)]
mod runtime_manifest_tests;
#[cfg(test)]
mod wheels_tests;

pub use lifecycle::SearxngSidecar;

use crate::services::agent_local::types_tools::SearchResult;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    lifecycle::search(query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    lifecycle::prepare_on_startup(app);
}
