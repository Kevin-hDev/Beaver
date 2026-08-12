use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::process::Stdio;

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

    crate::services::process_tree::configure_tokio(&mut cmd);

    cmd.kill_on_drop(true)
        .spawn()
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
    let python = ["python3", "python"]
        .into_iter()
        .find_map(|candidate| which::which(candidate).ok())
        .ok_or_else(|| "runtime de test indisponible".to_string())?;
    let mut command = tokio::process::Command::new(python);
    command
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    crate::services::process_tree::configure_tokio(&mut command);
    command
        .spawn()
        .map_err(|_| "fixture SearXNG indisponible".to_string())
}

pub fn startup_log_hint() -> Option<String> {
    let body = std::fs::read_to_string(log_path()).ok()?;
    for marker in [
        "ModuleNotFoundError:",
        "ImportError:",
        "No module named",
        "secret_key",
    ] {
        if let Some(line) = body.lines().rev().find(|line| line.contains(marker)) {
            return Some(sanitize_log_hint(line));
        }
    }
    None
}

fn sanitize_log_hint(line: &str) -> String {
    let mut out = String::new();
    for ch in line.chars() {
        if ch.is_control() {
            out.push(' ');
        } else {
            out.push(ch);
        }
        if out.chars().count() >= 180 {
            break;
        }
    }
    out.trim().to_string()
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
    fn sanitize_log_hint_removes_control_chars() {
        let hint = sanitize_log_hint("ModuleNotFoundError:\nNo module named 'x'");
        assert!(!hint.contains('\n'));
        assert!(hint.contains("ModuleNotFoundError:"));
    }
}
