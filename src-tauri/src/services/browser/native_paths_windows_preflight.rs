use super::cef_preflight::CefPreflightError;
use super::cef_unavailable::CefUnavailableCategory;
use super::native_paths::{
    windows_application_module, RuntimeFiles, MAX_RUNTIME_FILE_BYTES, WINDOWS_RELEASE_MODULE,
    WINDOWS_RUNTIME_FILES,
};
use std::path::{Path, PathBuf};

pub(super) fn resolve_runtime_files(executable: &Path) -> Result<RuntimeFiles, CefPreflightError> {
    let helper = canonicalize(executable)?;
    ensure_private_regular_file(&helper)?;
    let root = helper.parent().ok_or_else(invalid_runtime)?;
    let root = canonicalize(root)?;
    if !helper.starts_with(&root) {
        return Err(invalid_runtime());
    }

    let application_module = windows_application_module(&helper).ok_or_else(invalid_runtime)?;
    let module = canonicalize(&root.join(application_module))?;
    if !module.starts_with(&root) {
        return Err(invalid_runtime());
    }
    ensure_private_regular_file(&module)?;

    for relative in WINDOWS_RUNTIME_FILES {
        if relative == WINDOWS_RELEASE_MODULE {
            continue;
        }
        let candidate = canonicalize(&root.join(relative))?;
        if !candidate.starts_with(&root) {
            return Err(invalid_runtime());
        }
        ensure_private_regular_file(&candidate)?;
    }
    Ok(RuntimeFiles { helper })
}

fn canonicalize(path: &Path) -> Result<PathBuf, CefPreflightError> {
    dunce::canonicalize(path)
        .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))
}

fn ensure_private_regular_file(path: &Path) -> Result<(), CefPreflightError> {
    let metadata = path
        .symlink_metadata()
        .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))?;
    (metadata.file_type().is_file()
        && metadata.len() > 0
        && metadata.len() <= MAX_RUNTIME_FILE_BYTES)
        .then_some(())
        .ok_or_else(invalid_runtime)
}

fn invalid_runtime() -> CefPreflightError {
    CefPreflightError::deterministic(CefUnavailableCategory::Object)
}
