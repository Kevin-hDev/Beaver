use std::path::{Path, PathBuf};

use crate::services::ollama_lifecycle;

pub(crate) fn find_binary_in(dir: &Path) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    let in_bin = dir.join("bin").join(name);
    if in_bin.exists() {
        return Some(in_bin);
    }
    let at_root = dir.join(name);
    if at_root.exists() {
        return Some(at_root);
    }
    None
}

pub(crate) fn write_version_file(bundle_dir: &Path, version: &str) {
    let path = bundle_dir.join("VERSION");
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, version).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

pub fn read_version_file() -> Option<String> {
    let path = ollama_lifecycle::ollama_bundle_dir().join("VERSION");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
}

pub(crate) fn archives_to_download() -> Vec<&'static str> {
    if cfg!(target_os = "macos") {
        return vec!["ollama-darwin.tgz"];
    }
    if cfg!(target_os = "windows") {
        return vec!["ollama-windows-amd64.zip"];
    }

    use crate::services::gpu_detect::{self, GpuVendor};
    let gpu = gpu_detect::detect();
    eprintln!("[ollama] GPU détecté : {:?}", gpu);

    match gpu {
        GpuVendor::Amd => vec![
            "ollama-linux-amd64.tar.zst",
            "ollama-linux-amd64-rocm.tar.zst",
        ],
        _ => vec!["ollama-linux-amd64.tar.zst"],
    }
}

pub(crate) fn is_valid_semver(s: &str) -> bool {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$").unwrap());
    !s.is_empty() && RE.is_match(s)
}
