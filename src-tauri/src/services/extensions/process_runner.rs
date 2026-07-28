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
    temporary_directory: &Path,
    timeout: Duration,
) -> Result<(), String> {
    if !program.is_absolute()
        || !program.is_file()
        || !working_directory.is_absolute()
        || !working_directory.is_dir()
        || !temporary_directory.is_absolute()
        || !temporary_directory.is_dir()
        || arguments.len() > MAX_ARGUMENTS
        || arguments
            .iter()
            .any(|argument| argument.to_string_lossy().chars().count() > MAX_ARGUMENT_CHARS)
    {
        return Err("Commande d'installation invalide.".to_string());
    }
    let path = std::env::var_os("PATH")
        .filter(|value| valid_environment_value(value))
        .ok_or_else(|| "Environnement d'installation invalide.".to_string())?;
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .env("PATH", path)
        .env("TMPDIR", temporary_directory)
        .env("TMP", temporary_directory)
        .env("TEMP", temporary_directory);
    #[cfg(windows)]
    if let Some(system_root) =
        std::env::var_os("SystemRoot").filter(|value| valid_environment_value(value))
    {
        command.env("SystemRoot", system_root);
    }
    let mut child = command
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

fn valid_environment_value(value: &std::ffi::OsStr) -> bool {
    let text = value.to_string_lossy();
    text.chars().count() <= MAX_ARGUMENT_CHARS && !text.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_environment_is_bounded_and_single_line() {
        assert!(valid_environment_value(std::ffi::OsStr::new(
            "/usr/bin:/bin"
        )));
        assert!(!valid_environment_value(std::ffi::OsStr::new(
            "/usr/bin\nunsafe"
        )));
        assert!(!valid_environment_value(std::ffi::OsStr::new(
            &"a".repeat(MAX_ARGUMENT_CHARS + 1),
        )));
    }
}
