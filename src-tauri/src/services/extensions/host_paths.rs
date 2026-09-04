use std::path::PathBuf;
use tauri::Manager;

#[cfg(all(test, windows))]
#[path = "host_paths_windows_tests.rs"]
mod windows_tests;

pub struct HostPaths {
    pub node: PathBuf,
    pub script: PathBuf,
    pub directory: PathBuf,
}

impl HostPaths {
    pub(super) fn ui_builder(&self) -> PathBuf {
        self.directory.join("ui-build.mjs")
    }
}

pub fn resolve(app: &tauri::AppHandle) -> Result<HostPaths, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("extension-host");
    #[cfg(debug_assertions)]
    let manifest = {
        let prepared = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("extension-host");
        if prepared.join("host.mjs").is_file() {
            prepared
        } else {
            manifest
        }
    };
    let bundled = app
        .path()
        .resource_dir()
        .ok()
        .map(|root| root.join("resources").join("extension-host"));
    let directory = bundled
        .filter(|path| path.join("host.mjs").is_file())
        .unwrap_or(manifest);
    let directory = node_compatible_path(directory);
    let node_name = if cfg!(windows) { "node.exe" } else { "node" };
    let bundled_node = directory.join("runtime").join(node_name);
    let node = node_compatible_path(if bundled_node.is_file() {
        bundled_node
    } else {
        which::which(node_name)
            .or_else(|_| which::which("node"))
            .map_err(|_| "Runtime Node.js indisponible.".to_string())?
    });
    let script = directory.join("host.mjs");
    if !script.is_file() {
        return Err("Hôte d'extensions indisponible.".to_string());
    }
    let ui_builder = directory.join("ui-build.mjs");
    if !ui_builder.is_file() {
        return Err("Builder d'interface indisponible.".to_string());
    }
    Ok(HostPaths {
        node,
        script,
        directory,
    })
}

pub(super) fn node_compatible_path(path: PathBuf) -> PathBuf {
    dunce::simplified(&path).to_path_buf()
}
