use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use super::error_codes as errors;

const MAX_STARTUP_LOG_BYTES: u64 = 16 * 1024;

fn log_path() -> PathBuf {
    data_dir().join("logs").join("searxng-sidecar.log")
}

pub fn recover_orphan_sidecar(
    deadline: Instant,
    cancel: &crate::services::work_registry::ServiceWorkCancellation,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(errors::SHUTTING_DOWN.to_string());
    }
    let outcome = super::process_receipt::store()
        .recover_and_reap_with(deadline, || cancel.is_cancelled())
        .map_err(|_| errors::PROCESS_STATE_UNAVAILABLE.to_string())?;
    if cancel.is_cancelled() {
        return Err(errors::SHUTTING_DOWN.to_string());
    }
    match outcome {
        super::process_receipt::RecoveryOutcome::Blocked => {
            Err(errors::PROCESS_STATE_UNAVAILABLE.to_string())
        }
        _ => Ok(()),
    }
}

pub async fn spawn(
    python: &Path,
    source: &Path,
    settings: &Path,
    port: u16,
) -> Result<tokio::process::Child, String> {
    let log_dir = data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let stderr = super::private_file::create_private(&log_path())
        .map_err(|_| errors::LOG_UNAVAILABLE.to_string())?;
    let mut cmd = tokio::process::Command::new(python);
    cmd.args(["-m", "searx.webapp"])
        .current_dir(source)
        .env("SEARXNG_SETTINGS_PATH", settings)
        .env("SEARXNG_BIND_ADDRESS", "127.0.0.1")
        .env("SEARXNG_PORT", port.to_string())
        .env("SEARXNG_DEBUG", "0")
        .env("SEARXNG_LIMITER", "false")
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr));

    if let Some(path) = super::compat::python_path()? {
        cmd.env("PYTHONPATH", path);
    }

    cmd.kill_on_drop(true);
    crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut cmd,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await
    .map_err(|_| errors::START_FAILED.to_string())
}

pub(super) async fn stable_identity(
    pid: u32,
    deadline: tokio::time::Instant,
    cancel: &crate::services::work_registry::ServiceWorkCancellation,
) -> Result<crate::services::owned_process::OwnedProcessIdentity, String> {
    stable_identity_with(pid, deadline, cancel, |pid| {
        crate::services::owned_process::OwnedProcess::identity(pid).map_err(|_| ())
    })
    .await
}

pub(super) async fn stable_identity_with(
    pid: u32,
    deadline: tokio::time::Instant,
    cancel: &crate::services::work_registry::ServiceWorkCancellation,
    mut identity: impl FnMut(u32) -> Result<crate::services::owned_process::OwnedProcessIdentity, ()>,
) -> Result<crate::services::owned_process::OwnedProcessIdentity, String> {
    let mut previous = None;
    let mut stable_observations = 0;
    while tokio::time::Instant::now() < deadline {
        if cancel.is_cancelled() {
            return Err(errors::SHUTTING_DOWN.to_string());
        }
        let current = identity(pid).map_err(|_| errors::START_FAILED.to_string())?;
        if previous == Some(current) {
            stable_observations += 1;
            if stable_observations == 2 {
                return Ok(current);
            }
        } else {
            stable_observations = 0;
        }
        previous = Some(current);
        tokio::select! {
            _ = tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + Duration::from_millis(10))) => {}
            _ = cancel.cancelled() => return Err(errors::SHUTTING_DOWN.to_string()),
        }
    }
    Err(errors::START_FAILED.to_string())
}

pub async fn kill_child_process(mut child: tokio::process::Child) {
    let pid = child.id().unwrap_or_default();
    if let Ok(Some(_)) = child.try_wait() {
        let _ = super::process_receipt::store().remove();
        return;
    }
    ::log::info!("[searxng] kill sidecar pid={pid}");
    crate::services::process_tree::terminate_tokio(
        &mut child,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await;
    if child.try_wait().is_ok_and(|status| status.is_some()) {
        let _ = super::process_receipt::store().remove();
    }
}

#[cfg(test)]
pub async fn spawn_test_fixture() -> Result<tokio::process::Child, String> {
    let python = crate::services::test_runtime::python()?;
    let mut command = tokio::process::Command::new(python);
    command
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await
    .map_err(|_| "fixture SearXNG indisponible".to_string())
}

pub fn startup_log_hint() -> Option<String> {
    let bytes = read_log_tail(&log_path())?;
    let body = String::from_utf8_lossy(&bytes);
    classify_log_hint(&body).map(str::to_string)
}

pub(super) fn classify_log_hint(body: &str) -> Option<&'static str> {
    for (marker, category) in [
        ("ModuleNotFoundError:", "module-not-found"),
        ("ImportError:", "import-error"),
        ("No module named", "module-not-found"),
        ("secret_key", "invalid-secret-key"),
    ] {
        if body.lines().rev().any(|line| line.contains(marker)) {
            return Some(category);
        }
    }
    None
}

pub(super) fn read_log_tail(path: &Path) -> Option<Vec<u8>> {
    super::private_file::read_tail(path, MAX_STARTUP_LOG_BYTES).ok()
}
