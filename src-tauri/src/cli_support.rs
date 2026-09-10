//! Façade unique de la bibliothèque pour le binaire CLI `beaver`.
//! Tout accès CLI vers la bibliothèque passe ici ; `services` reste privé.

pub use crate::commands::app_update::{AppUpdateInfo, UpdateCheck};
pub use crate::services::config::read_config;
pub use crate::services::paths::data_dir;
pub use crate::services::vault::vault_path;

pub const APP_LOG_MAX_BYTES: u64 = crate::services::app_log::MAX_FILE_BYTES as u64;
pub const WAKEUP_LOG_MAX_LINES: usize = crate::services::automations::history_max_lines();
pub const WAKEUP_LOG_MAX_LINE_BYTES: usize =
    crate::services::automations::history_max_line_bytes();

pub fn version_gt(remote: &str, local: &str) -> bool {
    crate::commands::app_update::version_gt(remote, local)
}

pub async fn check_app_update_detailed() -> Result<UpdateCheck, String> {
    crate::commands::app_update::check_app_update_detailed().await
}

pub async fn download_and_launch_app_update(update: AppUpdateInfo) -> Result<(), String> {
    crate::commands::app_update_cli::download_and_launch(update).await
}

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

/// Renvoie uniquement des dossiers directement enfants de `root` afin que la
/// garde de suppression CLI puisse exiger l'égalité de leur parent canonique.
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

pub fn remove_tool_results(
    root: &std::path::Path,
    paths: &[std::path::PathBuf],
) -> CliRemovalOutcome {
    let outcome = crate::services::agent_local::tool_result_budget::remove_results(root, paths);
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
