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
pub(crate) const MAX_OLLAMA_PATH_COMPONENTS: usize = 64;
#[allow(dead_code)]
pub(crate) const PROCESS_REAP_FALLBACK_TIMEOUT: Duration =
    crate::app_exit::OLLAMA_REAP_RESERVE_TIMEOUT;
pub(crate) const OWNED_START_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
pub(crate) const MAX_PROBE_PORT_ATTEMPTS: usize = 3;
#[allow(dead_code)]
pub(crate) const MAX_PROBE_RESPONSE_BYTES: usize = 4 * 1024;
#[allow(dead_code)]
pub(crate) const PROBE_DEFAULT_PORT: u16 = 11_434;
#[allow(dead_code)]
pub(crate) const PROBE_CONNECT_TIMEOUT: Duration = Duration::from_millis(250);
#[allow(dead_code)]
pub(crate) const PROBE_ENDPOINT_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[allow(dead_code)]
pub(crate) const MAX_PROBE_SOCKET_RECORDS: usize = 1024;
