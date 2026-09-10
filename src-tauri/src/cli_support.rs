//! Façade unique de la bibliothèque pour le binaire CLI `beaver`.
//! Tout accès CLI vers la bibliothèque passe ici ; `services` reste privé.

pub use crate::services::config::read_config;
pub use crate::services::paths::data_dir;
pub use crate::services::vault::vault_path;
