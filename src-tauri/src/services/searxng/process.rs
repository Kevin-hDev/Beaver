use crate::services::paths::data_dir;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::process::Stdio;

const MAX_STARTUP_LOG_BYTES: u64 = 16 * 1024;

fn pid_path() -> PathBuf {
    data_dir().join("searxng-sidecar.pid")
}

fn log_path() -> PathBuf {
    data_dir().join("logs").join("searxng-sidecar.log")
}

pub fn save_pid(pid: u32) {
    let tmp = pid_path().with_extension("tmp");
    if std::fs::write(&tmp, pid.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, pid_path());
    }
}

pub fn clear_pid_file() {
    let _ = std::fs::remove_file(pid_path());
}

pub fn kill_orphan_sidecar() {
    let Some(pid) = read_saved_pid() else { return };
    clear_pid_file();
    if !is_searxng_process(pid) {
        ::log::warn!("[searxng] pid={pid} ignoré");
        return;
    }
    ::log::info!("[searxng] orphelin détecté pid={pid}, kill");
    crate::services::process_tree::kill(pid, crate::services::process_tree::ProcessKind::Searxng);
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

pub async fn kill_child_process(mut child: tokio::process::Child) {
    let pid = child.id().unwrap_or_default();
    if let Ok(Some(_)) = child.try_wait() {
        clear_pid_file();
        return;
    }
    ::log::info!("[searxng] kill sidecar pid={pid}");
    crate::services::process_tree::terminate_tokio(
        &mut child,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await;
    clear_pid_file();
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

fn read_saved_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_path()).ok()?;
    let pid = content.trim().parse::<u32>().ok()?;
    (pid >= 2).then_some(pid)
}

fn is_searxng_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "command="])
            .output();
        output
            .ok()
            .map(|o| process_text_matches(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        let query =
            format!("(Get-CimInstance Win32_Process -Filter \"ProcessId = {pid}\").CommandLine");
        let mut command = crate::services::background_command::new("powershell");
        let output = command.args(["-NoProfile", "-Command", &query]).output();
        output
            .ok()
            .map(|o| process_text_matches(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(false)
    }
}

pub(crate) fn process_text_matches(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("searxng-sidecar") && lower.contains("searx.webapp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_match_requires_sidecar_and_webapp() {
        assert!(process_text_matches(
            "python -m searx.webapp /searxng-sidecar/.venv"
        ));
        assert!(!process_text_matches("python -m searx.webapp"));
        assert!(!process_text_matches("searxng-sidecar unrelated"));
    }

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
