use super::source_validation::NpmSource;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct NpmRunner {
    node: PathBuf,
    cli: PathBuf,
}

impl NpmRunner {
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, String> {
        let paths = super::host_paths::resolve(app)?;
        let node = paths
            .node
            .canonicalize()
            .map_err(|_| "Runtime Node.js indisponible.".to_string())?;
        let cli = resolve_cli(&paths.directory, &node)?;
        Ok(Self { node, cli })
    }

    pub fn install_package(&self, prefix: &Path, source: &NpmSource) -> Result<PathBuf, String> {
        let cache = prefix.join(".npm-cache");
        let mut arguments = self.common_arguments("install", prefix, &cache);
        arguments.extend([
            OsString::from("--package-lock=false"),
            OsString::from("--save=false"),
            OsString::from("--"),
            OsString::from(&source.locator),
        ]);
        let result = self.run(prefix, arguments);
        let _ = std::fs::remove_dir_all(&cache);
        result?;
        let package_root = prefix
            .join("node_modules")
            .join(package_path(&source.package_name));
        if !package_root.is_dir() {
            return Err("Package npm installé introuvable.".to_string());
        }
        Ok(package_root)
    }

    pub fn install_dependencies(&self, root: &Path) -> Result<(), String> {
        let cache = root.join(".npm-cache");
        let command = if root.join("package-lock.json").is_file()
            || root.join("npm-shrinkwrap.json").is_file()
        {
            "ci"
        } else {
            "install"
        };
        let mut arguments = self.common_arguments(command, root, &cache);
        if command == "install" {
            arguments.extend([
                OsString::from("--package-lock=false"),
                OsString::from("--save=false"),
            ]);
        }
        let result = self.run(root, arguments);
        let _ = std::fs::remove_dir_all(&cache);
        result
    }

    pub(super) fn common_arguments(
        &self,
        command: &str,
        prefix: &Path,
        cache: &Path,
    ) -> Vec<OsString> {
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
            OsString::from("--prefix"),
            prefix.as_os_str().to_owned(),
            OsString::from("--cache"),
            cache.as_os_str().to_owned(),
        ]
    }

    fn run(&self, root: &Path, arguments: Vec<OsString>) -> Result<(), String> {
        super::process_runner::run(&self.node, &arguments, root, INSTALL_TIMEOUT)
    }

    #[cfg(test)]
    pub(super) fn for_test(node: PathBuf, cli: PathBuf) -> Self {
        Self { node, cli }
    }
}

pub(super) fn resolve_cli(host_directory: &Path, node: &Path) -> Result<PathBuf, String> {
    let bundled = host_directory.join("runtime/npm/bin/npm-cli.js");
    if bundled.is_file() {
        return bundled
            .canonicalize()
            .map_err(|_| "Gestionnaire npm indisponible.".to_string());
    }
    let bin = node
        .parent()
        .ok_or_else(|| "Gestionnaire npm indisponible.".to_string())?;
    #[cfg(windows)]
    let inferred = bin.join("node_modules/npm/bin/npm-cli.js");
    #[cfg(not(windows))]
    let inferred = bin.join("../lib/node_modules/npm/bin/npm-cli.js");
    let discovered = which::which("npm")
        .ok()
        .and_then(|path| path.canonicalize().ok());
    for candidate in [Some(inferred), discovered].into_iter().flatten() {
        if let Ok(candidate) = candidate.canonicalize() {
            if candidate.is_file()
                && candidate.file_name().and_then(|name| name.to_str()) == Some("npm-cli.js")
            {
                return Ok(candidate);
            }
        }
    }
    Err("Gestionnaire npm indisponible.".to_string())
}

pub(super) fn package_path(name: &str) -> PathBuf {
    name.split('/').collect()
}
