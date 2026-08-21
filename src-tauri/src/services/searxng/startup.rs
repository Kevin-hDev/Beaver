use crate::services::work_registry::ServiceWorkCancellation;
use std::time::Duration;

const READY_POLL: Duration = Duration::from_millis(250);
const READY_REQUEST_TIMEOUT: Duration = Duration::from_millis(500);

pub(super) async fn wait_until_ready(
    base_url: &str,
    child: &mut tokio::process::Child,
    cancel: &ServiceWorkCancellation,
    deadline: tokio::time::Instant,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{base_url}/healthz");
    while tokio::time::Instant::now() < deadline {
        if cancel.is_cancelled() {
            return Err(shutdown_error());
        }
        if let Ok(Some(status)) = child.try_wait() {
            let hint = super::process::startup_log_hint().unwrap_or_default();
            ::log::warn!("[searxng] startup exited status={status} hint={hint}");
            return Err(super::error_codes::START_FAILED.to_string());
        }
        tokio::select! {
            _ = tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + READY_POLL)) => {}
            _ = cancel.cancelled() => return Err(shutdown_error()),
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        if let Ok(response) = client
            .get(&url)
            .timeout(remaining.min(READY_REQUEST_TIMEOUT))
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
        }
    }
    ::log::warn!("[searxng] startup readiness deadline elapsed");
    Err(super::error_codes::START_FAILED.to_string())
}

pub(super) async fn run_blocking<Operation>(operation: Operation) -> Result<(), String>
where
    Operation: FnOnce() -> Result<(), String> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| super::error_codes::OPERATION_INTERRUPTED.to_string())?
}

fn shutdown_error() -> String {
    super::error_codes::SHUTTING_DOWN.to_string()
}
