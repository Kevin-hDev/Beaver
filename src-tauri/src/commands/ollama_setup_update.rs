use super::ollama_bundle_utils::is_valid_semver;
use super::ollama_setup::{OllamaSetupProgress, start_manager_and_wait};
use crate::services::ollama_manager::OllamaManager;
use std::path::Path;
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn update_ollama_binary(
    version: String,
    on_progress: Channel<OllamaSetupProgress>,
    manager: tauri::State<'_, OllamaManager>,
) -> Result<(), String> {
    let version = version.trim_start_matches('v');
    if !is_valid_semver(version) {
        return Err("ollama-version-invalid".into());
    }

    let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
    let dest = paths.active;
    let staging = paths.update_staging;
    let backup = paths.backup;
    if backup.exists() {
        return Err("ollama-update-recovery-required".into());
    }
    remove_temp_dir(&staging)?;

    let cancel = CancellationToken::new();
    if let Err(error) = super::ollama_setup_install::install_ollama_to(
        manager.inner(),
        &staging,
        version,
        &on_progress,
        &cancel,
    )
    .await
    {
        let _ = remove_temp_dir(&staging);
        send_status(&on_progress, "error");
        return Err(error);
    }

    manager.stop_and_wait(Instant::now() + Duration::from_secs(15)).await
        .map_err(|code| code.as_str().to_string())?;
    let had_backup = swap_installation(&dest, &staging, &backup)?;
    send_status(&on_progress, "restarting");

    if start_manager_and_wait(manager.inner(), &on_progress, &cancel)
        .await
        .is_ok()
    {
        if had_backup {
            let _ = remove_temp_dir(&backup);
        }
        ::log::info!("[ollama-update] mis à jour vers v{version}");
        return Ok(());
    }

    rollback_after_failed_restart(manager.inner(), &on_progress, &dest, &backup, had_backup).await
}

fn swap_installation(dest: &Path, staging: &Path, backup: &Path) -> Result<bool, String> {
    let had_backup = dest.exists();
    if had_backup {
        std::fs::rename(dest, backup).map_err(|error| {
            ::log::error!("[ollama-update] backup: {error}");
            "ollama-update-error".to_string()
        })?;
    }
    if let Err(error) = std::fs::rename(staging, dest) {
        if had_backup {
            let _ = std::fs::rename(backup, dest);
        }
        ::log::error!("[ollama-update] swap: {error}");
        return Err("ollama-update-error".into());
    }
    Ok(had_backup)
}

async fn rollback_after_failed_restart(
    manager: &OllamaManager,
    on_progress: &Channel<OllamaSetupProgress>,
    dest: &Path,
    backup: &Path,
    had_backup: bool,
) -> Result<(), String> {
    manager.stop_and_wait(Instant::now() + Duration::from_secs(15)).await
        .map_err(|code| code.as_str().to_string())?;
    if !had_backup {
        return Err("ollama-restart-error".into());
    }

    let failed = dest.with_file_name("ollama-bundle-failed");
    remove_temp_dir(&failed)?;
    restore_previous_install(dest, backup, &failed)?;

    let cancel = CancellationToken::new();
    let restored = start_manager_and_wait(manager, on_progress, &cancel)
        .await
        .is_ok();
    if restored {
        let _ = remove_temp_dir(&failed);
        Err("ollama-restart-error".into())
    } else {
        Err("ollama-rollback-error".into())
    }
}

fn restore_previous_install(dest: &Path, backup: &Path, failed: &Path) -> Result<(), String> {
    std::fs::rename(dest, failed).map_err(|_| "ollama-rollback-error".to_string())?;
    if std::fs::rename(backup, dest).is_ok() {
        return Ok(());
    }
    if std::fs::rename(failed, dest).is_err() {
        ::log::error!("[ollama-update] failed installation recovery failed");
    }
    Err("ollama-rollback-error".into())
}

fn remove_temp_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path).map_err(|error| {
        ::log::error!("[ollama-update] temporary cleanup: {error}");
        "ollama-update-error".to_string()
    })
}

fn send_status(channel: &Channel<OllamaSetupProgress>, status: &str) {
    let _ = channel.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: status.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_keeps_previous_install_as_backup() {
        let root = tempfile::tempdir().unwrap();
        let dest = root.path().join("ollama-bundle");
        let staging = root.path().join("ollama-bundle-staging");
        let backup = root.path().join("ollama-bundle-old");
        std::fs::create_dir(&dest).unwrap();
        std::fs::write(dest.join("version"), "old").unwrap();
        std::fs::create_dir(&staging).unwrap();
        std::fs::write(staging.join("version"), "new").unwrap();

        assert!(swap_installation(&dest, &staging, &backup).unwrap());
        assert_eq!(
            std::fs::read_to_string(dest.join("version")).unwrap(),
            "new"
        );
        assert_eq!(
            std::fs::read_to_string(backup.join("version")).unwrap(),
            "old"
        );
    }

    #[test]
    fn failed_backup_restore_puts_the_new_install_back_in_place() {
        let root = tempfile::tempdir().unwrap();
        let dest = root.path().join("ollama-bundle");
        let missing_backup = root.path().join("ollama-bundle-old");
        let failed = root.path().join("ollama-bundle-failed");
        std::fs::create_dir(&dest).unwrap();
        std::fs::write(dest.join("version"), "new").unwrap();

        assert!(restore_previous_install(&dest, &missing_backup, &failed).is_err());
        assert_eq!(
            std::fs::read_to_string(dest.join("version")).unwrap(),
            "new"
        );
        assert!(!failed.exists());
    }
}
