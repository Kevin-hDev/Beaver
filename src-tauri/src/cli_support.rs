//! Façade unique de la bibliothèque pour le binaire CLI `beaver`.
//! Tout accès CLI vers la bibliothèque passe ici ; `services` reste privé.

pub use crate::services::config::read_config;
pub use crate::services::paths::data_dir;
pub use crate::services::vault::vault_path;

pub fn ollama_bundle_dir(root: &std::path::Path) -> std::path::PathBuf {
    crate::services::paths::ollama_paths(root).active
}

pub fn ollama_models_dir() -> Option<std::path::PathBuf> {
    crate::services::ollama_manager::cli_access::models_directory_path()
}

pub fn ollama_process_name_matches(name: &str) -> bool {
    crate::services::ollama_manager::cli_access::process_name_matches(name)
}
