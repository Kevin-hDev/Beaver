use crate::services::{paths::data_dir, process_tree};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

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
    if !is_forecast_process(pid) {
        eprintln!("[forecast] pid={pid} n'est plus le sidecar, ignoré");
        return;
    }
    if !is_forecast_process(pid) {
        eprintln!("[forecast] pid={pid} changé entre check et kill, abandon");
        return;
    }
    eprintln!("[forecast] orphelin détecté pid={pid}, kill");
    process_tree::kill(pid, process_tree::ProcessKind::Forecast);
}

pub fn kill_child_process(mut child: Child) {
    let pid = child.id();
    eprintln!("[forecast] kill sidecar pid={pid}");
    process_tree::kill(pid, process_tree::ProcessKind::Forecast);
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(_)) = child.try_wait() {
            eprintln!("[forecast] sidecar arrêté");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
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
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &query])
            .output();
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
