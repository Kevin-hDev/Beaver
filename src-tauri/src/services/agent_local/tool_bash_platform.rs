use tokio::process::Command;

#[cfg(all(test, windows))]
#[path = "tool_bash_platform_windows_tests.rs"]
mod windows_tests;

#[cfg(unix)]
const TERMINATION_GRACE_MS: u64 = 50;

pub fn configure_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW,
        };
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }
}

#[cfg(windows)]
pub fn powershell_executable() -> Result<std::path::PathBuf, String> {
    system32_file(&["WindowsPowerShell", "v1.0", "powershell.exe"])
        .ok_or_else(|| "Shell utilisateur indisponible.".to_string())
}

#[cfg(windows)]
fn system32_file(components: &[&str]) -> Option<std::path::PathBuf> {
    let root = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_absolute())?;
    let mut path = root.join("System32");
    for component in components {
        if component.is_empty()
            || component.contains('/')
            || component.contains('\\')
            || *component == ".."
        {
            return None;
        }
        path.push(component);
    }
    path.is_file().then_some(path)
}

pub async fn terminate_process_tree(pid: u32) {
    #[cfg(unix)]
    {
        let Ok(pid) = i32::try_from(pid) else {
            return;
        };
        let group = -pid;
        // SAFETY: the child was placed in a dedicated process group before spawn.
        unsafe {
            libc::kill(group, libc::SIGTERM);
        }
        tokio::time::sleep(std::time::Duration::from_millis(TERMINATION_GRACE_MS)).await;
        // SAFETY: the same validated process-group id is used for forced cleanup.
        unsafe {
            libc::kill(group, libc::SIGKILL);
        }
    }

    #[cfg(windows)]
    {
        let pid = pid.to_string();
        let Some(taskkill) = system32_file(&["taskkill.exe"]) else {
            return;
        };
        let _ = crate::services::background_command::new_tokio(taskkill)
            .args(["/PID", &pid, "/T", "/F"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;
    }
}
