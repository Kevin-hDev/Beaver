use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MAX_ARGUMENTS: usize = 48;
const MAX_ARGUMENT_CHARS: usize = 4_096;
const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn run(
    program: &Path,
    arguments: &[OsString],
    working_directory: &Path,
    timeout: Duration,
) -> Result<(), String> {
    if !program.is_absolute()
        || !program.is_file()
        || !working_directory.is_absolute()
        || !working_directory.is_dir()
        || arguments.len() > MAX_ARGUMENTS
        || arguments
            .iter()
            .any(|argument| argument.to_string_lossy().chars().count() > MAX_ARGUMENT_CHARS)
    {
        return Err("Commande d'installation invalide.".to_string());
    }
    let mut child = Command::new(program)
        .args(arguments)
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_remove("NODE_OPTIONS")
        .env_remove("NPM_CONFIG_SCRIPT_SHELL")
        .env_remove("npm_config_script_shell")
        .spawn()
        .map_err(|_| "Commande d'installation indisponible.".to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => return Err("Commande d'installation échouée.".to_string()),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                crate::services::process_tree::kill(
                    child.id(),
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                let _ = child.wait();
                return Err("Commande d'installation expirée.".to_string());
            }
            Err(_) => {
                crate::services::process_tree::kill(
                    child.id(),
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                let _ = child.wait();
                return Err("Commande d'installation interrompue.".to_string());
            }
        }
    }
}
