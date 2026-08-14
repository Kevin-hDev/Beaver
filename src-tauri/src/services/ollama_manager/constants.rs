use std::time::Duration;

pub(crate) const OLLAMA_WORK_CAPACITY: usize = 1;
#[allow(dead_code)]
pub(crate) const MAX_DURABLE_DOCUMENT_BYTES: usize = 4 * 1024;
#[allow(dead_code)]
// Cette fenêtre couvre seulement les verrous antivirus transitoires, sans attente ouverte.
pub(crate) const WINDOWS_SHARING_RETRY_TIMEOUT: Duration = Duration::from_secs(2);
#[allow(dead_code)]
pub(crate) const WINDOWS_SHARING_RETRY_INTERVAL: Duration = Duration::from_millis(50);

pub(crate) const MAX_OLLAMA_ENV_ENTRIES: usize = 256;
pub(crate) const MAX_OLLAMA_ENV_KEY_UNITS: usize = 256;
pub(crate) const MAX_OLLAMA_ENV_VALUE_UNITS: usize = 8_192;
pub(crate) const MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES: usize = 65_536;
pub(crate) const MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16: usize = 32_767;
