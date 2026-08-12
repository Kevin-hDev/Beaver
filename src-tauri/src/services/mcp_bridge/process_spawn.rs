use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::BufReader;
use tokio::process::{Child, Command};
use zeroize::Zeroizing;

use super::process_manager::ProcessHandle;

pub(super) async fn spawn_program(
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
        // Le contenu n'était jamais exposé ni journalisé. Le rediriger vers
        // null supprime un lecteur détaché sans perdre de diagnostic public.
        .stderr(Stdio::null())
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
    let Some(stdin) = child.stdin.take() else {
        reap_failed_spawn(&mut child).await;
        return Err("stdin indisponible".to_string());
    };
    let Some(stdout) = child.stdout.take() else {
        reap_failed_spawn(&mut child).await;
        return Err("stdout indisponible".to_string());
    };

    let handle = ProcessHandle {
        stdin: Arc::new(tokio::sync::Mutex::new(Some(stdin))),
        reader: Arc::new(tokio::sync::Mutex::new(BufReader::new(stdout))),
        request_lock: Arc::new(tokio::sync::Mutex::new(())),
        initialized: Arc::new(tokio::sync::OnceCell::new()),
    };
    Ok((child, handle))
}

async fn reap_failed_spawn(child: &mut Child) {
    crate::services::process_tree::terminate_tokio(
        child,
        crate::services::process_tree::ProcessKind::Mcp,
    )
    .await;
}
