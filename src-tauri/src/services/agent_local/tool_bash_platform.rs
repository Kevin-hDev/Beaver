use tokio::process::Command;

const TERMINATION_GRACE_MS: u64 = 50;

pub fn configure_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
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
        let _ = Command::new("taskkill")
            .args(["/PID", &pid, "/T", "/F"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;
    }
}
