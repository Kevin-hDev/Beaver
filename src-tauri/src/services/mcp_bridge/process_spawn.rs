use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use zeroize::Zeroizing;

use super::process_manager::ProcessHandle;

pub(super) fn spawn_program(
    program_path: &Path,
    args: &[String],
    env_tokens: &[(String, Zeroizing<String>)],
) -> Result<(Child, ProcessHandle), String> {
    let safe_env = super::process_env::safe_env()?;
    let mut command = Command::new(program_path);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    for (key, value) in safe_env {
        command.env(key, value);
    }
    for (key, value) in env_tokens {
        command.env(key, value.as_str());
    }
    crate::services::process_tree::configure_tokio(&mut command);

    let mut child = command
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "impossible de démarrer le connecteur MCP".to_string())?;
    let stdin = child.stdin.take().ok_or("stdin indisponible")?;
    let stdout = child.stdout.take().ok_or("stdout indisponible")?;

    if let Some(mut stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut buffer = [0_u8; 4096];
            while matches!(stderr.read(&mut buffer).await, Ok(size) if size > 0) {}
        });
    }

    let handle = ProcessHandle {
        stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
        reader: Arc::new(tokio::sync::Mutex::new(BufReader::new(stdout))),
        request_lock: Arc::new(tokio::sync::Mutex::new(())),
    };
    Ok((child, handle))
}
