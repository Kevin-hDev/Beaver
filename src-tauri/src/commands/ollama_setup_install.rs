use super::ollama_bundle_utils::archives_to_download;
use super::ollama_setup::OllamaSetupProgress;
use crate::services::ollama_manager::{
    InstallOutcome, InstallRequest, OllamaManager, OllamaVersion,
};
use std::ffi::OsString;
use std::path::Path;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

const _: fn(&Path) -> Option<std::path::PathBuf> = super::ollama_bundle_utils::find_binary_in;
const _: fn(&Path, &str) = super::ollama_bundle_utils::write_version_file;

pub(crate) async fn install_ollama_to(
    manager: &OllamaManager,
    dest: &Path,
    version: &str,
    _on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let version = OllamaVersion::parse(version).map_err(|_| "ollama-version-invalid")?;
    let data_dir = dest
        .parent()
        .ok_or("ollama-storage-unavailable")?
        .to_path_buf();
    let mut paths = crate::services::paths::ollama_paths(&data_dir);
    paths.active = dest.to_path_buf();
    paths.install_staging = dest.with_file_name(format!(
        "{}-install-staging",
        dest.file_name()
            .and_then(|name| name.to_str())
            .ok_or("ollama-storage-unavailable")?
    ));
    let names = archives_to_download();
    let manifest =
        crate::services::ollama_manager::release_source::fetch_manifest(version.clone(), &names)
            .await
            .map_err(|code| code.as_str().to_string())?;
    let request = InstallRequest {
        paths,
        version: Some(version),
        manifest: Some(manifest),
        inherited_environment: std::env::vars_os().collect::<Vec<(OsString, OsString)>>(),
        inherited_cwd: std::env::current_dir().map_err(|_| "ollama-storage-unavailable")?,
        cancellation: cancel.clone(),
        deadline: None,
        #[cfg(test)]
        local_archives: None,
    };
    match manager
        .install(request)
        .await
        .map_err(|code| code.as_str().to_string())?
    {
        InstallOutcome::Installed { .. } => Ok(()),
        InstallOutcome::Preparing => Err("ollama-install-incomplete".to_string()),
    }
}
