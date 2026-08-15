#![allow(dead_code)]

use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(crate) fn extract_zip(
    archive: &Path,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    crate::services::ollama_manager::extract_archive_overlay(
        archive,
        destination,
        "ollama-windows-amd64.zip",
        cancellation,
    )
    .map_err(|code| code.as_str().to_string())
}
