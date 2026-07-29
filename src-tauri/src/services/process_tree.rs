#[cfg(windows)]
use std::process::Command;
#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Copy)]
pub enum ProcessKind {
    ExtensionInstaller,
    Forecast,
    ForecastRuntime,
    Ollama,
    Searxng,
}

impl ProcessKind {
    fn label(self) -> &'static str {
        match self {
            Self::ExtensionInstaller => "extension-installer",
            Self::Forecast => "forecast",
            Self::ForecastRuntime => "forecast-runtime",
            Self::Ollama => "ollama",
            Self::Searxng => "searxng",
        }
    }
}

pub fn kill(pid: u32, kind: ProcessKind) {
    let label = kind.label();
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .output();
        eprintln!("[{label}] tree-kill Windows pid={pid}");
    }

    #[cfg(unix)]
    {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let children = collect_children(&sys, Pid::from_u32(pid));
        for &child_pid in children.iter().rev() {
            let raw = child_pid.as_u32() as i32;
            eprintln!("[{label}] kill child pid={raw}");
            // SAFETY: `raw` is only passed as an OS pid; libc::kill touches no Rust memory.
            unsafe {
                libc::kill(raw, libc::SIGTERM);
            }
        }
        // SAFETY: `pid` is only passed as an OS pid; libc::kill touches no Rust memory.
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            // SAFETY: signal 0 checks process existence and does not touch memory.
            if unsafe { libc::kill(pid as i32, 0) != 0 } {
                eprintln!("[{label}] arbre pid={pid} arrêté");
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        for &child_pid in children.iter().rev() {
            // SAFETY: child pids are only passed as OS pids; libc::kill touches no Rust memory.
            unsafe {
                libc::kill(child_pid.as_u32() as i32, libc::SIGKILL);
            }
        }
        // SAFETY: `pid` is only passed as an OS pid; libc::kill touches no Rust memory.
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
        eprintln!("[{label}] SIGKILL arbre pid={pid}");
    }
}

#[cfg(unix)]
const MAX_CHILDREN: usize = 256;
#[cfg(unix)]
const MAX_DEPTH: u32 = 10;

#[cfg(unix)]
fn collect_children(sys: &System, parent: Pid) -> Vec<Pid> {
    let mut result = Vec::new();
    collect_children_inner(sys, parent, &mut result, 0);
    result
}

#[cfg(unix)]
fn collect_children_inner(sys: &System, parent: Pid, result: &mut Vec<Pid>, depth: u32) {
    if depth >= MAX_DEPTH || result.len() >= MAX_CHILDREN {
        return;
    }
    for (pid, process) in sys.processes() {
        if result.len() >= MAX_CHILDREN {
            return;
        }
        if process.parent() == Some(parent) {
            result.push(*pid);
            collect_children_inner(sys, *pid, result, depth + 1);
        }
    }
}
