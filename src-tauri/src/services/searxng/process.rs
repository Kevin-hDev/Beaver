use crate::services::paths::data_dir;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

const MAX_STARTUP_LOG_BYTES: u64 = 16 * 1024;

fn log_path() -> PathBuf {
    data_dir().join("logs").join("searxng-sidecar.log")
}

pub fn recover_orphan_sidecar() -> Result<(), String> {
    match super::process_receipt::store()
        .recover_and_reap(Instant::now() + Duration::from_secs(3))
        .map_err(|_| "SearXNG: état processus illisible".to_string())?
    {
        super::process_receipt::RecoveryOutcome::Blocked => {
            Err("SearXNG: état processus illisible".to_string())
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
    let stderr =
        std::fs::File::create(log_path()).map_err(|_| "SearXNG: log indisponible".to_string())?;
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
    .map_err(|_| "SearXNG: démarrage impossible".to_string())
}

pub(super) async fn stable_identity(
    pid: u32,
) -> Result<crate::services::owned_process::OwnedProcessIdentity, String> {
    let mut previous = None;
    let mut stable_observations = 0;
    for _ in 0..20 {
        let current = crate::services::owned_process::OwnedProcess::identity(pid)
            .map_err(|_| "SearXNG: démarrage impossible".to_string())?;
        if previous == Some(current) {
            stable_observations += 1;
            if stable_observations == 2 {
                return Ok(current);
            }
        } else {
            stable_observations = 0;
        }
        previous = Some(current);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err("SearXNG: démarrage impossible".to_string())
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

fn classify_log_hint(body: &str) -> Option<&'static str> {
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

fn read_log_tail(path: &Path) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let start = length.saturating_sub(MAX_STARTUP_LOG_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.take(MAX_STARTUP_LOG_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_log_hint_exposes_only_a_fixed_category() {
        let root = tempfile::tempdir().unwrap();
        let log = root.path().join("sidecar.log");
        std::fs::write(&log, "ModuleNotFoundError: secret/path").unwrap();
        let body = String::from_utf8_lossy(&read_log_tail(&log).unwrap()).to_string();
        assert!(body.contains("secret/path"));
        assert_eq!(classify_log_hint(&body), Some("module-not-found"));
    }

    #[test]
    fn startup_diagnostic_reads_only_the_bounded_tail() {
        let root = tempfile::tempdir().unwrap();
        let log = root.path().join("sidecar.log");
        let mut body = vec![b'x'; MAX_STARTUP_LOG_BYTES as usize * 2];
        body.extend_from_slice(b"\nModuleNotFoundError: bounded-tail");
        std::fs::write(&log, body).unwrap();

        let tail = read_log_tail(&log).unwrap();
        assert!(tail.len() <= MAX_STARTUP_LOG_BYTES as usize);
        assert!(String::from_utf8_lossy(&tail).contains("bounded-tail"));
    }
}
