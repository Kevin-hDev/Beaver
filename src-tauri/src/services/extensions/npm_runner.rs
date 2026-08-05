use super::source_validation::NpmSource;
use super::OperationFailure;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
const REGISTRY: &str = "https://registry.npmjs.org/";

#[derive(Clone)]
pub struct NpmRunner {
    node: PathBuf,
    cli: PathBuf,
}

impl NpmRunner {
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, OperationFailure> {
        let paths =
            super::host_paths::resolve(app).map_err(|_| OperationFailure::RuntimeUnavailable)?;
        let node = paths
            .node
            .canonicalize()
            .map_err(|_| OperationFailure::RuntimeUnavailable)?;
        let cli = resolve_cli(&paths.directory, &node)
            .map_err(|_| OperationFailure::RuntimeUnavailable)?;
        Ok(Self::new(node, cli))
    }

    pub fn install_package(
        &self,
        prefix: &Path,
        source: &NpmSource,
    ) -> Result<PathBuf, OperationFailure> {
        let workspace = prepare_workspace(prefix).map_err(|_| OperationFailure::StorageFailed)?;
        let mut arguments = self.common_arguments("install", &workspace);
        arguments.extend([
            OsString::from("--package-lock=false"),
            OsString::from("--save=false"),
            OsString::from("--"),
            OsString::from(&source.locator),
        ]);
        let result = self.run(prefix, &workspace, arguments);
        cleanup_workspace(&workspace).map_err(|_| OperationFailure::StorageFailed)?;
        result?;
        let package_root = prefix
            .join("node_modules")
            .join(package_path(&source.package_name));
        if !package_root.is_dir() {
            return Err(OperationFailure::PackageInvalid);
        }
        Ok(package_root)
    }

    pub fn install_dependencies(&self, root: &Path) -> Result<(), OperationFailure> {
        let workspace = prepare_workspace(root).map_err(|_| OperationFailure::StorageFailed)?;
        let config = super::npm_environment::ProjectConfig::neutralize(root)
            .map_err(|_| OperationFailure::StorageFailed)?;
        let command = if root.join("package-lock.json").is_file()
            || root.join("npm-shrinkwrap.json").is_file()
        {
            "ci"
        } else {
            "install"
        };
        let mut arguments = self.common_arguments(command, &workspace);
        if command == "install" {
            arguments.extend([
                OsString::from("--package-lock=false"),
                OsString::from("--save=false"),
            ]);
        }
        let result = self.run(root, &workspace, arguments);
        let restore = config.restore();
        let cleanup = cleanup_workspace(&workspace);
        restore.map_err(|_| OperationFailure::StorageFailed)?;
        cleanup.map_err(|_| OperationFailure::StorageFailed)?;
        result
    }

    pub(super) fn common_arguments(&self, command: &str, workspace: &Path) -> Vec<OsString> {
        vec![
            self.cli.as_os_str().to_owned(),
            OsString::from(command),
            OsString::from("--ignore-scripts"),
            OsString::from("--omit=dev"),
            OsString::from("--no-audit"),
            OsString::from("--no-fund"),
            OsString::from("--no-bin-links"),
            OsString::from("--workspaces=false"),
            OsString::from("--progress=false"),
            OsString::from("--cache"),
            workspace.join("cache").into_os_string(),
            OsString::from("--registry"),
            OsString::from(REGISTRY),
            OsString::from("--replace-registry-host=always"),
            OsString::from("--strict-ssl=true"),
            OsString::from("--userconfig"),
            workspace.join("userconfig").into_os_string(),
            OsString::from("--globalconfig"),
            workspace.join("globalconfig").into_os_string(),
        ]
    }

    fn run(
        &self,
        root: &Path,
        workspace: &Path,
        arguments: Vec<OsString>,
    ) -> Result<(), OperationFailure> {
        super::process_runner::run(
            &self.node,
            &arguments,
            root,
            &workspace.join("tmp"),
            INSTALL_TIMEOUT,
        )
        .map_err(OperationFailure::from)
    }

    #[cfg(test)]
    pub(super) fn for_test(node: PathBuf, cli: PathBuf) -> Self {
        Self::new(node, cli)
    }

    #[cfg(test)]
    pub(super) fn paths_for_test(&self) -> (&Path, &Path) {
        (&self.node, &self.cli)
    }

    fn new(node: PathBuf, cli: PathBuf) -> Self {
        Self {
            node: super::host_paths::node_compatible_path(node),
            cli: super::host_paths::node_compatible_path(cli),
        }
    }
}

pub(super) fn resolve_cli(host_directory: &Path, _node: &Path) -> Result<PathBuf, String> {
    let bundled = host_directory.join("runtime/npm/bin/npm-cli.js");
    if bundled.is_file() {
        return bundled
            .canonicalize()
            .map(super::host_paths::node_compatible_path)
            .map_err(|_| "Gestionnaire npm indisponible.".to_string());
    }
    #[cfg(test)]
    let bin = _node
        .parent()
        .ok_or_else(|| "Gestionnaire npm indisponible.".to_string())?;
    #[cfg(test)]
    #[cfg(windows)]
    let inferred = bin.join("node_modules/npm/bin/npm-cli.js");
    #[cfg(test)]
    #[cfg(not(windows))]
    let inferred = bin.join("../lib/node_modules/npm/bin/npm-cli.js");
    #[cfg(test)]
    if let Ok(candidate) = inferred.canonicalize() {
        if candidate.is_file()
            && candidate.file_name().and_then(|name| name.to_str()) == Some("npm-cli.js")
        {
            return Ok(super::host_paths::node_compatible_path(candidate));
        }
    }
    Err("Gestionnaire npm indisponible.".to_string())
}

fn prepare_workspace(root: &Path) -> Result<PathBuf, String> {
    let workspace = root.join(".npm-cache");
    if workspace.exists() {
        std::fs::remove_dir_all(&workspace)
            .map_err(|_| "Cache npm impossible à nettoyer.".to_string())?;
    }
    crate::services::private_store::ensure_private_dir(&workspace.join("cache"))
        .map_err(|_| "Cache npm indisponible.".to_string())?;
    crate::services::private_store::ensure_private_dir(&workspace.join("tmp"))
        .map_err(|_| "Cache npm indisponible.".to_string())?;
    for name in ["userconfig", "globalconfig"] {
        crate::services::private_store::atomic_write(&workspace.join(name), b"")
            .map_err(|_| "Configuration npm indisponible.".to_string())?;
    }
    Ok(workspace)
}

fn cleanup_workspace(workspace: &Path) -> Result<(), String> {
    std::fs::remove_dir_all(workspace).map_err(|_| "Cache npm impossible à nettoyer.".to_string())
}

pub(super) fn package_path(name: &str) -> PathBuf {
    name.split('/').collect()
}
