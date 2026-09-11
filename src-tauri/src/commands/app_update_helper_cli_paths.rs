use super::app_update_helper::{current_install_directory_for, install_error};
use std::path::{Path, PathBuf};

pub(crate) fn cli_update_paths_for(executable: &Path) -> Result<(PathBuf, PathBuf), String> {
    let executable = std::fs::canonicalize(executable).map_err(|_| install_error())?;
    let working_directory = current_install_directory_for(&executable)?;
    #[cfg(target_os = "macos")]
    let resource_root = working_directory
        .parent()
        .map(|contents| contents.join("Resources"))
        .ok_or_else(install_error)?
        .canonicalize()
        .map_err(|_| install_error())?;
    #[cfg(not(target_os = "macos"))]
    let resource_root = cli_resource_directory()?;
    Ok((resource_root, working_directory))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn cli_resource_directory() -> Result<PathBuf, String> {
    let package = tauri::utils::PackageInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION")
            .parse()
            .map_err(|_| install_error())?,
        authors: env!("CARGO_PKG_AUTHORS"),
        description: env!("CARGO_PKG_DESCRIPTION"),
        crate_name: env!("CARGO_PKG_NAME"),
    };
    tauri::utils::platform::resource_dir(&package, &tauri::utils::Env::default())
        .map_err(|_| install_error())
}
