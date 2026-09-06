use super::source_validation::NpmSource;
use super::OperationFailure;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::install_signal::InstallSignal;
use super::npm_workspace::{cleanup as cleanup_workspace, prepare as prepare_workspace};

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
        cancellation: &impl InstallSignal,
    ) -> Result<PathBuf, OperationFailure> {
        let workspace = prepare_workspace(prefix).map_err(|_| OperationFailure::StorageFailed)?;
        let mut remaining = INSTALL_TIMEOUT;
        let mut arguments = self.common_arguments("install", &workspace);
        arguments.extend([
            OsString::from("--package-lock-only"),
            OsString::from("--save-exact"),
            OsString::from("--"),
            OsString::from(&source.locator),
        ]);
        let result = self
            .run(prefix, &workspace, arguments, &mut remaining, cancellation)
            .and_then(|()| {
                cancellation.lock_dependencies(prefix)?;
                self.run(
                    prefix,
                    &workspace,
                    self.common_arguments("ci", &workspace),
                    &mut remaining,
                    cancellation,
                )
            });
        if result == Err(OperationFailure::CleanupFailed) {
            return result.map(|_| prefix.to_path_buf());
        }
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

    pub fn install_dependencies(
        &self,
        root: &Path,
        cancellation: &impl InstallSignal,
    ) -> Result<(), OperationFailure> {
        let workspace = prepare_workspace(root).map_err(|_| OperationFailure::StorageFailed)?;
        let mut remaining = INSTALL_TIMEOUT;
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
        if command == "ci" {
            cancellation.lock_dependencies(root)?;
        }
        if command == "install" {
            arguments.extend([OsString::from("--package-lock-only")]);
        }
        let result = self
            .run(root, &workspace, arguments, &mut remaining, cancellation)
            .and_then(|()| {
                if command == "install" {
                    cancellation.lock_dependencies(root)?;
                    self.run(
                        root,
                        &workspace,
                        self.common_arguments("ci", &workspace),
                        &mut remaining,
                        cancellation,
                    )
                } else {
                    Ok(())
                }
            });
        if result == Err(OperationFailure::CleanupFailed) {
            config.retain();
            return result;
        }
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
        remaining: &mut Duration,
        cancellation: &impl InstallSignal,
    ) -> Result<(), OperationFailure> {
        super::process_runner::run(
            &self.node,
            &arguments,
            root,
            &workspace.join("tmp"),
            remaining,
            cancellation,
        )
        .map_err(OperationFailure::from)
    }

    #[cfg(any(test, feature = "e2e"))]
    pub(super) fn for_test(node: PathBuf, cli: PathBuf) -> Self {
        Self::new(node, cli)
    }

    #[cfg(all(test, windows))]
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

pub(super) fn package_path(name: &str) -> PathBuf {
    name.split('/').collect()
}

pub(super) use super::npm_paths::resolve_cli;
