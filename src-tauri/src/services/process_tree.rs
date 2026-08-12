use std::process::{Child, Command};
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::process::Stdio;
#[cfg(unix)]
#[path = "process_tree_unix.rs"]
mod unix;

const GRACEFUL_STOP_TIMEOUT: Duration = Duration::from_millis(500);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, Copy)]
pub enum ProcessKind {
    AgentShell,
    ExtensionHost,
    ExtensionInstaller,
    Forecast,
    ForecastRuntime,
    Mcp,
    Ollama,
    Searxng,
    Terminal,
    UpdateHelper,
}

#[cfg(all(test, windows))]
#[path = "process_tree_windows_tests.rs"]
mod windows_tests;

impl ProcessKind {
    fn label(self) -> &'static str {
        match self {
            Self::AgentShell => "agent-shell",
            Self::ExtensionHost => "extension-host",
            Self::ExtensionInstaller => "extension-installer",
            Self::Forecast => "forecast",
            Self::ForecastRuntime => "forecast-runtime",
            Self::Mcp => "mcp",
            Self::Ollama => "ollama",
            Self::Searxng => "searxng",
            Self::Terminal => "terminal",
            Self::UpdateHelper => "update-helper",
        }
    }
}

pub fn configure(command: &mut Command) {
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(target_os = "linux")]
    unsafe {
        command.pre_exec(configure_linux_parent_death);
    }
    #[cfg(windows)]
    crate::services::background_command::configure(command);
}

pub fn configure_tokio(command: &mut tokio::process::Command) {
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(target_os = "linux")]
    unsafe {
        command.pre_exec(configure_linux_parent_death);
    }
    #[cfg(windows)]
    crate::services::background_command::configure_tokio(command);
}

pub fn terminate(child: &mut Child, kind: ProcessKind) {
    if child.try_wait().ok().flatten().is_some() {
        crate::services::owned_process::release(child.id());
        return;
    }
    let pid = child.id();
    signal_tree(pid, false);
    if wait_for_child(child, GRACEFUL_STOP_TIMEOUT) {
        crate::services::owned_process::release(pid);
        ::log::info!("[{}] arbre pid={pid} arrêté", kind.label());
        return;
    }
    force_tree(pid);
    let _ = child.kill();
    let _ = child.wait();
    crate::services::owned_process::release(pid);
    ::log::warn!("[{}] arrêt forcé arbre pid={pid}", kind.label());
}

pub async fn terminate_tokio(child: &mut tokio::process::Child, kind: ProcessKind) {
    if child.try_wait().ok().flatten().is_some() {
        if let Some(pid) = child.id() {
            crate::services::owned_process::release(pid);
        }
        return;
    }
    let Some(pid) = child.id() else {
        return;
    };
    signal_tree(pid, false);
    let deadline = tokio::time::Instant::now() + GRACEFUL_STOP_TIMEOUT;
    while tokio::time::Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            crate::services::owned_process::release(pid);
            ::log::info!("[{}] arbre pid={pid} arrêté", kind.label());
            return;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    force_tree(pid);
    let _ = child.start_kill();
    let _ = child.wait().await;
    crate::services::owned_process::release(pid);
    ::log::warn!("[{}] arrêt forcé arbre pid={pid}", kind.label());
}

pub fn kill(pid: u32, kind: ProcessKind) {
    if pid < 2 {
        return;
    }
    signal_tree(pid, false);
    #[cfg(unix)]
    {
        std::thread::sleep(Duration::from_millis(100));
        force_tree(pid);
    }
    crate::services::owned_process::release(pid);
    ::log::info!("[{}] arrêt arbre orphelin pid={pid}", kind.label());
}

#[cfg(unix)]
fn force_tree(pid: u32) {
    signal_tree(pid, true);
}

#[cfg(windows)]
fn force_tree(_pid: u32) {}

fn wait_for_child(child: &mut Child, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return true;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    false
}

#[cfg(unix)]
fn signal_tree(pid: u32, force: bool) {
    if !crate::services::owned_process::signal_is_safe(pid) {
        ::log::warn!("[process] identité macOS changée pid={pid}, signal ignoré");
        return;
    }
    let Ok(raw_pid) = i32::try_from(pid) else {
        return;
    };
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    let children = unix::collect_children(pid);
    // SAFETY: les identifiants proviennent de l'OS et aucun pointeur Rust n'est utilisé.
    unsafe {
        libc::kill(-raw_pid, signal);
        for child in children.iter().rev() {
            libc::kill(child.as_u32() as i32, signal);
        }
        libc::kill(raw_pid, signal);
    }
}

#[cfg(target_os = "linux")]
fn configure_linux_parent_death() -> std::io::Result<()> {
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::getppid() } == 1 {
        unsafe { libc::raise(libc::SIGKILL) };
    }
    Ok(())
}

#[cfg(windows)]
fn signal_tree(pid: u32, _force: bool) {
    let Some(system_root) = std::env::var_os("SystemRoot") else {
        return;
    };
    let executable = std::path::PathBuf::from(system_root)
        .join("System32")
        .join("taskkill.exe");
    if !executable.is_absolute() || !executable.is_file() {
        return;
    }
    let _ = crate::services::background_command::new(executable)
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn terminate_reaps_child_without_three_second_delay() {
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        configure(&mut command);
        let mut child = command.spawn().unwrap();
        let started = std::time::Instant::now();

        terminate(&mut child, ProcessKind::ForecastRuntime);

        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(child.try_wait().unwrap().is_some());
    }
}
