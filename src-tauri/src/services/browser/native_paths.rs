use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
const FRAMEWORK_BINARY: &str = "Chromium Embedded Framework.framework/Chromium Embedded Framework";
#[cfg(target_os = "macos")]
const HELPER_BINARY: &str = "Beaver Helper.app/Contents/MacOS/Beaver Helper";
#[cfg(any(test, target_os = "windows"))]
pub(super) const WINDOWS_RUNTIME_FILES: [&str; 23] = [
    "cl-go-dash.dll",
    "chrome_elf.dll",
    "d3dcompiler_47.dll",
    "dxcompiler.dll",
    "dxil.dll",
    "libEGL.dll",
    "libGLESv2.dll",
    "libcef.dll",
    "v8_context_snapshot.bin",
    "vk_swiftshader.dll",
    "vk_swiftshader_icd.json",
    "vulkan-1.dll",
    "chrome_100_percent.pak",
    "chrome_200_percent.pak",
    "icudtl.dat",
    "resources.pak",
    "locales/de.pak",
    "locales/en-US.pak",
    "locales/es.pak",
    "locales/fr.pak",
    "locales/it.pak",
    "locales/ja.pak",
    "locales/zh-CN.pak",
];
#[cfg(any(test, target_os = "windows"))]
pub(super) const MAX_RUNTIME_FILE_BYTES: u64 = 512 * 1024 * 1024;
#[cfg(any(test, target_os = "windows"))]
const WINDOWS_RELEASE_BOOTSTRAP: &str = "cl-go-dash.exe";
#[cfg(any(test, target_os = "windows"))]
pub(super) const WINDOWS_RELEASE_MODULE: &str = "cl-go-dash.dll";
#[cfg(any(test, target_os = "windows"))]
const WINDOWS_DEVELOPMENT_BOOTSTRAP: &str = "cl_go_dash_lib.exe";
#[cfg(any(test, target_os = "windows"))]
const WINDOWS_DEVELOPMENT_MODULE: &str = "cl_go_dash_lib.dll";

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeFiles {
    #[cfg(target_os = "macos")]
    pub(super) framework: PathBuf,
    pub(super) helper: PathBuf,
}

#[cfg(target_os = "macos")]
pub(super) fn framework_candidates(
    executable: &Path,
    downloaded_cef_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::with_capacity(2);
    if let Some(root) = bundle_framework_root(executable) {
        candidates.push(root.join(FRAMEWORK_BINARY));
    }
    if let Some(downloaded) = downloaded_cef_dir {
        candidates.push(downloaded.join(FRAMEWORK_BINARY));
    }
    candidates
}

#[cfg(target_os = "macos")]
pub(super) fn helper_executable(executable: &Path) -> Option<PathBuf> {
    Some(
        bundle_contents(executable)?
            .join("Frameworks")
            .join(HELPER_BINARY),
    )
}

#[cfg(all(test, target_os = "macos"))]
pub(super) fn resolve_runtime_files(
    executable: &Path,
    downloaded_cef_dir: Option<&Path>,
) -> Option<RuntimeFiles> {
    let bundle_root = bundle_framework_root(executable)?;
    let canonical_bundle_root = bundle_root.canonicalize().ok()?;
    let helper = helper_executable(executable)?.canonicalize().ok()?;
    if !helper.is_file() || !helper.starts_with(&canonical_bundle_root) {
        return None;
    }

    let candidates = framework_candidates(executable, downloaded_cef_dir);
    let mut roots = vec![canonical_bundle_root];
    if let Some(downloaded) = downloaded_cef_dir {
        roots.push(downloaded.canonicalize().ok()?);
    }
    for (candidate, root) in candidates.into_iter().zip(roots) {
        let Ok(framework) = candidate.canonicalize() else {
            continue;
        };
        if framework.is_file() && framework.starts_with(root) {
            return Some(RuntimeFiles { framework, helper });
        }
    }
    None
}

#[cfg(test)]
pub(super) fn resolve_windows_runtime_files(executable: &Path) -> Option<PathBuf> {
    let helper = executable.canonicalize().ok()?;
    if !private_regular_file(&helper) {
        return None;
    }
    let root = helper.parent()?.canonicalize().ok()?;
    if !helper.starts_with(&root) {
        return None;
    }
    let application_module = windows_application_module(&helper)?;
    let canonical_module = root.join(application_module).canonicalize().ok()?;
    if !canonical_module.starts_with(&root) || !private_regular_file(&canonical_module) {
        return None;
    }
    for relative in WINDOWS_RUNTIME_FILES {
        if relative == WINDOWS_RELEASE_MODULE {
            continue;
        }
        let candidate = root.join(relative);
        let canonical = candidate.canonicalize().ok()?;
        if !canonical.starts_with(&root) || !private_regular_file(&canonical) {
            return None;
        }
    }
    Some(helper)
}

#[cfg(target_os = "macos")]
pub(super) fn bundle_framework_root(executable: &Path) -> Option<PathBuf> {
    Some(bundle_contents(executable)?.join("Frameworks"))
}

#[cfg(any(test, target_os = "windows"))]
pub(super) fn windows_application_module(executable: &Path) -> Option<&'static str> {
    let name = executable.file_name()?.to_str()?;
    if name.eq_ignore_ascii_case(WINDOWS_RELEASE_BOOTSTRAP) {
        return Some(WINDOWS_RELEASE_MODULE);
    }
    if name.eq_ignore_ascii_case(WINDOWS_DEVELOPMENT_BOOTSTRAP) {
        return Some(WINDOWS_DEVELOPMENT_MODULE);
    }
    None
}

#[cfg(test)]
fn private_regular_file(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|metadata| {
        metadata.file_type().is_file()
            && metadata.len() > 0
            && metadata.len() <= MAX_RUNTIME_FILE_BYTES
    })
}

#[cfg(target_os = "macos")]
fn bundle_contents(executable: &Path) -> Option<&Path> {
    let macos = executable.parent()?;
    if macos.file_name()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()? != "Contents" {
        return None;
    }
    let app_bundle = contents.parent()?;
    (app_bundle.extension()? == "app").then_some(contents)
}
