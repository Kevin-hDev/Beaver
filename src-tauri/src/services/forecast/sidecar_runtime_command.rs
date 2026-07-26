use crate::services::process_tree;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const REMOVED_PYTHON_ENV: &[&str] = &[
    "CONDA_PREFIX",
    "PIP_CERT",
    "PIP_CLIENT_CERT",
    "PIP_EXTRA_INDEX_URL",
    "PIP_FIND_LINKS",
    "PIP_INDEX_URL",
    "PIP_NO_VERIFY",
    "PIP_TRUSTED_HOST",
    "PYTHONHOME",
    "PYTHONPATH",
    "VIRTUAL_ENV",
];

pub(super) fn configure_pip(command: &mut Command, cache: &Path) {
    for key in REMOVED_PYTHON_ENV {
        command.env_remove(key);
    }
    command
        .env("PIP_CACHE_DIR", cache)
        .env("PIP_CONFIG_FILE", null_device())
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("PIP_INDEX_URL", "https://pypi.org/simple")
        .env("PIP_NO_INPUT", "1")
        .env("PIP_REQUIRE_HASHES", "1")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("PYTHONUNBUFFERED", "1");
}

pub(super) fn harden_python(command: &mut Command) {
    for key in REMOVED_PYTHON_ENV {
        command.env_remove(key);
    }
    command
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("PYTHONUNBUFFERED", "1");
}

pub(super) fn run_cancellable(
    command: &mut Command,
    cancel: &CancellationToken,
    message: &str,
) -> Result<(), String> {
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = command.spawn().map_err(|_| message.to_string())?;
    loop {
        if cancel.is_cancelled() {
            process_tree::kill(child.id(), process_tree::ProcessKind::ForecastRuntime);
            let _ = child.wait();
            return Err("cancelled".to_string());
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) | Err(_) => return Err(message.to_string()),
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

fn null_device() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

#[cfg(test)]
mod tests {
    use super::{configure_pip, harden_python};
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn pip_uses_only_the_approved_index_and_cache() {
        let mut command = Command::new("python");
        configure_pip(&mut command, Path::new("cache"));
        let env = command.get_envs().collect::<Vec<_>>();
        assert!(env.iter().any(|(key, value)| {
            *key == "PIP_REQUIRE_HASHES" && value.is_some_and(|value| value == "1")
        }));
        assert!(env.iter().any(|(key, value)| {
            *key == "PIP_INDEX_URL" && value.is_some_and(|value| value == "https://pypi.org/simple")
        }));
    }

    #[test]
    fn python_does_not_resolve_user_packages() {
        let mut command = Command::new("python");
        harden_python(&mut command);
        assert!(command.get_envs().any(|(key, value)| {
            key == "PYTHONNOUSERSITE" && value.is_some_and(|value| value == "1")
        }));
    }
}
