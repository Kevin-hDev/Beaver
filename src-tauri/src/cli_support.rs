//! Façade unique de la bibliothèque pour le binaire CLI `beaver`.
//! Tout accès CLI vers la bibliothèque passe ici ; `services` reste privé.

pub use crate::services::config::read_config;
pub use crate::services::paths::data_dir;
pub use crate::services::vault::vault_path;

pub const APP_LOG_MAX_BYTES: u64 = crate::services::app_log::MAX_FILE_BYTES as u64;
pub const WAKEUP_LOG_MAX_LINES: usize = crate::services::scheduler::log::MAX_LINES;
pub const WAKEUP_LOG_MAX_LINE_BYTES: usize = crate::services::scheduler::log::MAX_LOG_LINE_BYTES;

pub fn ollama_bundle_dir(root: &std::path::Path) -> std::path::PathBuf {
    crate::services::paths::ollama_paths(root).active
}

pub fn ollama_models_dir() -> Option<std::path::PathBuf> {
    crate::services::ollama_manager::cli_access::models_directory_path()
}

pub fn ollama_process_name_matches(name: &str) -> bool {
    crate::services::ollama_manager::cli_access::process_name_matches(name)
}

pub fn ollama_bundle_is_valid(root: &std::path::Path) -> bool {
    crate::services::ollama_manager::cli_access::bundle_is_valid(root)
}

pub fn ollama_bundle_receipt_tmp_path(root: &std::path::Path) -> std::path::PathBuf {
    crate::services::ollama_manager::cli_access::bundle_receipt_tmp_path(root)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliSupportError;

pub struct CliRemovalOutcome {
    pub removed: usize,
    pub failed: Vec<std::path::PathBuf>,
}

pub fn abandoned_ollama_staging_dirs(
    root: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, CliSupportError> {
    crate::services::ollama_manager::cli_access::abandoned_staging_dirs(root)
        .map_err(|_| CliSupportError)
}

pub fn old_tool_results(
    root: &std::path::Path,
    now: std::time::SystemTime,
) -> Result<Vec<(std::path::PathBuf, u64)>, CliSupportError> {
    crate::services::agent_local::tool_result_budget::old_results_in(root, now)
        .map_err(|_| CliSupportError)
}

pub fn remove_tool_results(paths: &[std::path::PathBuf]) -> CliRemovalOutcome {
    let outcome = crate::services::agent_local::tool_result_budget::remove_results(paths);
    CliRemovalOutcome {
        removed: outcome.removed,
        failed: outcome.failed.into_iter().map(|(path, _)| path).collect(),
    }
}

pub fn remove_abandoned_ollama_staging(
    root: &std::path::Path,
    directories: &[std::path::PathBuf],
) -> CliRemovalOutcome {
    let outcome =
        crate::services::ollama_manager::cli_access::remove_abandoned_staging(root, directories);
    CliRemovalOutcome {
        removed: outcome.removed,
        failed: outcome.failed,
    }
}
