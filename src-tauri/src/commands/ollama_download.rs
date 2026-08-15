#![allow(dead_code)]

use super::ollama_setup::OllamaSetupProgress;
use crate::services::ollama_manager::OllamaArchive;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub async fn download_file(
    archive: &OllamaArchive,
    destination: &std::path::Path,
    on_progress: &Channel<OllamaSetupProgress>,
    cancellation: &CancellationToken,
    status: &str,
) -> Result<(), String> {
    crate::services::ollama_manager::download_archive(archive, destination, cancellation)
        .await
        .map_err(|code| code.as_str().to_string())?;
    let _ = on_progress.send(OllamaSetupProgress {
        completed: archive.expected_size,
        total: archive.expected_size,
        status: status.to_owned(),
    });
    Ok(())
}
