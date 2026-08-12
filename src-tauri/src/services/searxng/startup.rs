use crate::services::work_registry::ServiceWorkCancellation;
use std::time::Duration;

const READY_ATTEMPTS: usize = 40;
const READY_POLL: Duration = Duration::from_millis(250);
const READY_REQUEST_TIMEOUT: Duration = Duration::from_millis(500);
const PUBLIC_START_ERROR: &str = "SearXNG: démarrage impossible";

pub(super) async fn wait_until_ready(
    base_url: &str,
    child: &mut tokio::process::Child,
    cancel: &ServiceWorkCancellation,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{base_url}/healthz");
    for _ in 0..READY_ATTEMPTS {
        if cancel.is_cancelled() {
            return Err(shutdown_error());
        }
        if let Ok(Some(status)) = child.try_wait() {
            let hint = super::process::startup_log_hint().unwrap_or_default();
            ::log::warn!("[searxng] startup exited status={status} hint={hint}");
            return Err(PUBLIC_START_ERROR.to_string());
        }
        tokio::select! {
            _ = tokio::time::sleep(READY_POLL) => {}
            _ = cancel.cancelled() => return Err(shutdown_error()),
        }
        if let Ok(response) = client.get(&url).timeout(READY_REQUEST_TIMEOUT).send().await {
            if response.status().is_success() {
                return Ok(());
            }
        }
    }
    ::log::warn!("[searxng] startup readiness deadline elapsed");
    Err(PUBLIC_START_ERROR.to_string())
}

pub(super) async fn run_blocking<Operation>(operation: Operation) -> Result<(), String>
where
    Operation: FnOnce() + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| "SearXNG: opération interrompue".to_string())
}

fn shutdown_error() -> String {
    "SearXNG: arrêt en cours".to_string()
}
