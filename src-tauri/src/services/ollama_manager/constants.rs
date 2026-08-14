use std::time::Duration;

pub(crate) const OLLAMA_WORK_CAPACITY: usize = 1;
#[allow(dead_code)]
pub(crate) const MAX_DURABLE_DOCUMENT_BYTES: usize = 4 * 1024;
#[allow(dead_code)]
// Cette fenêtre couvre seulement les verrous antivirus transitoires, sans attente ouverte.
pub(crate) const WINDOWS_SHARING_RETRY_TIMEOUT: Duration = Duration::from_secs(2);
#[allow(dead_code)]
pub(crate) const WINDOWS_SHARING_RETRY_INTERVAL: Duration = Duration::from_millis(50);
