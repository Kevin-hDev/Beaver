use std::path::Path;
use uuid::Uuid;

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::ollama_bundle_utils::{archives_to_download, find_binary_in, write_version_file};
use super::ollama_download::download_file;
use super::ollama_setup::OllamaSetupProgress;

pub(crate) async fn install_ollama_to(
    dest: &Path,
    version: &str,
    on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let archives = archives_to_download();
    if !dest.join("VERSION").exists() {
        let _ = std::fs::remove_dir_all(dest);
    }
    let result = install_archives_to(dest, version, on_progress, cancel, &archives).await;
    if let Err(err) = &result {
        if super::ollama_setup_cancel::is_cancelled_error(err) {
            cleanup_cancelled_install(dest);
        }
    }
    result
}

async fn install_archives_to(
    dest: &Path,
    version: &str,
    on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
    archives: &[&str],
) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| {
        eprintln!("[ollama-setup] mkdir {}: {e}", dest.display());
        "Impossible de créer le dossier d'installation".to_string()
    })?;

    let checksums = fetch_checksums(version, archives).await?;

    for (i, archive_name) in archives.iter().enumerate() {
        ensure_not_cancelled(cancel)?;
        let url = format!(
            "https://github.com/ollama/ollama/releases/download/v{}/{}",
            version, archive_name
        );

        let status = if i == 0 {
            "downloading"
        } else {
            "downloading-rocm"
        };
        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0,
            total: 0,
            status: status.into(),
        });

        let tmp =
            std::env::temp_dir().join(format!("cl-go-ollama-{}-{archive_name}", Uuid::new_v4()));
        if let Err(err) = download_file(&url, &tmp, on_progress, cancel, status).await {
            let _ = std::fs::remove_file(&tmp);
            let _ = std::fs::remove_dir_all(dest);
            return Err(err);
        }

        let expected = checksums
            .get(i)
            .ok_or_else(|| "checksum-not-found".to_string())?;
        ensure_not_cancelled(cancel)?;
        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0,
            total: 0,
            status: "verifying".into(),
        });
        if let Err(err) = super::ollama_checksum::verify_file_sha256(&tmp, expected, cancel) {
            let _ = std::fs::remove_file(&tmp);
            let _ = std::fs::remove_dir_all(dest);
            return Err(err);
        }

        ensure_not_cancelled(cancel)?;
        let _ = on_progress.send(OllamaSetupProgress {
            completed: 0,
            total: 0,
            status: "extracting".into(),
        });

        if let Err(err) = super::ollama_extract::extract_overlay(&tmp, dest, archive_name, cancel) {
            let _ = std::fs::remove_dir_all(dest);
            let _ = std::fs::remove_file(&tmp);
            return Err(err);
        }
        let _ = std::fs::remove_file(&tmp);
    }

    ensure_not_cancelled(cancel)?;
    let binary = find_binary_in(dest).ok_or_else(|| {
        let _ = std::fs::remove_dir_all(dest);
        "installation incomplète: binaire Ollama introuvable".to_string()
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755));
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("xattr")
            .args(["-d", "com.apple.quarantine"])
            .arg(&binary)
            .output();
        eprintln!("[ollama] quarantine attribute supprimé");
    }

    write_version_file(dest, version);
    eprintln!("[ollama-setup] installé v{version}: {}", binary.display());
    Ok(())
}

fn cleanup_cancelled_install(dest: &Path) {
    let _ = std::fs::remove_dir_all(dest);
    eprintln!("[ollama-setup] installation annulée, fichiers partiels supprimés");
}

fn ensure_not_cancelled(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(super::ollama_setup_cancel::cancelled_error());
    }
    Ok(())
}

async fn fetch_checksums(version: &str, archives: &[&str]) -> Result<Vec<String>, String> {
    let mut result = Vec::with_capacity(archives.len());
    for name in archives {
        match super::ollama_checksum::fetch_expected_hash(version, name).await {
            Ok(hash) => result.push(hash),
            Err(e) => {
                eprintln!("[ollama-setup] checksum unavailable for {name}: {e}");
                return Err("checksum-not-available".to_string());
            }
        }
    }
    Ok(result)
}
