use crate::services::{paths::data_dir, process_tree};
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;

fn pid_path() -> PathBuf {
    data_dir().join("chronos-sidecar.pid")
}

pub fn save_pid(pid: u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let tmp = pid_path().with_extension("tmp");
    if std::fs::write(&tmp, format!("{pid}:{now}")).is_ok() {
        let _ = std::fs::rename(&tmp, pid_path());
    }
}

pub fn clear_pid_file() {
    let _ = std::fs::remove_file(pid_path());
}

pub fn kill_orphan_sidecar() {
    let Some(pid) = read_saved_pid() else { return };
    clear_pid_file();
    let Some(pid) = verified_orphan_pid(pid, is_forecast_process) else {
        ::log::warn!("[forecast] pid={pid} n'est plus le sidecar, ignoré");
        return;
    };
    ::log::info!("[forecast] orphelin détecté pid={pid}, kill");
    process_tree::kill(pid, process_tree::ProcessKind::Forecast);
}

fn verified_orphan_pid(pid: u32, identity_probe: impl FnOnce(u32) -> bool) -> Option<u32> {
    identity_probe(pid).then_some(pid)
}

pub async fn kill_child_process(mut child: tokio::process::Child, pid: u32) {
    ::log::info!("[forecast] kill sidecar pid={pid}");
    process_tree::terminate_tokio(&mut child, process_tree::ProcessKind::Forecast).await;
}

fn read_saved_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_path()).ok()?;
    let pid = content.trim().split(':').next()?.parse::<u32>().ok()?;
    (pid >= 2).then_some(pid)
}

fn is_forecast_process(pid: u32) -> bool {
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
        let Ok(powershell) = crate::services::system_executable::powershell() else {
            return false;
        };
        let mut command = crate::services::background_command::new(powershell);
        let output = command.args(["-NoProfile", "-Command", &query]).output();
        output
            .ok()
            .map(|o| process_text_matches(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(false)
    }
}

fn process_text_matches(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("forecast-sidecar") && lower.contains("server.py")
}

#[cfg(test)]
pub(super) async fn spawn_test_fixture() -> Result<super::sidecar_spawn::PendingSidecar, String> {
    let python = crate::services::test_runtime::python()?;
    let mut command = tokio::process::Command::new(python);
    command
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let child = crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        process_tree::ProcessKind::Forecast,
    )
    .await
    .map_err(|_| "fixture Forecast indisponible".to_string())?;
    super::sidecar_spawn::PendingSidecar::new(child)
}

#[cfg(test)]
mod tests {
    #[test]
    fn orphan_decision_probes_identity_once() {
        let probes = std::cell::Cell::new(0);

        assert_eq!(
            super::verified_orphan_pid(42, |_| {
                probes.set(probes.get() + 1);
                true
            }),
            Some(42)
        );
        assert_eq!(probes.get(), 1);
    }
}
