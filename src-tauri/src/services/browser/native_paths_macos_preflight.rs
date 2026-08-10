use super::cef_preflight::CefPreflightError;
use super::cef_unavailable::CefUnavailableCategory;
use super::native_paths::{
    bundle_framework_root, framework_candidates, helper_executable, RuntimeFiles,
};
use std::io;
use std::path::{Path, PathBuf};

pub(super) fn resolve_runtime_files(
    executable: &Path,
    downloaded_cef_dir: Option<&Path>,
) -> Result<RuntimeFiles, CefPreflightError> {
    let bundle_root = bundle_framework_root(executable).ok_or_else(invalid_runtime)?;
    let canonical_bundle_root = canonicalize_required(&bundle_root)?;
    let helper =
        canonicalize_required(&helper_executable(executable).ok_or_else(invalid_runtime)?)?;
    ensure_contained_regular_file(&helper, &canonical_bundle_root)?;

    let candidates = framework_candidates(executable, downloaded_cef_dir);
    let bundled = candidates.first().ok_or_else(invalid_runtime)?;
    if let Some(framework) = resolve_candidate(bundled, &canonical_bundle_root)? {
        return Ok(RuntimeFiles { framework, helper });
    }
    if let Some(downloaded) = downloaded_cef_dir {
        if let Some(downloaded_root) = canonicalize_optional(downloaded)? {
            let candidate = candidates.last().ok_or_else(invalid_runtime)?;
            if let Some(framework) = resolve_candidate(candidate, &downloaded_root)? {
                return Ok(RuntimeFiles { framework, helper });
            }
        }
    }
    Err(invalid_runtime())
}

fn resolve_candidate(candidate: &Path, root: &Path) -> Result<Option<PathBuf>, CefPreflightError> {
    let Some(candidate) = canonicalize_optional(candidate)? else {
        return Ok(None);
    };
    ensure_contained_regular_file(&candidate, root)?;
    Ok(Some(candidate))
}

fn canonicalize_required(path: &Path) -> Result<PathBuf, CefPreflightError> {
    dunce::canonicalize(path)
        .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))
}

fn canonicalize_optional(path: &Path) -> Result<Option<PathBuf>, CefPreflightError> {
    match dunce::canonicalize(path) {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CefPreflightError::from_io(
            CefUnavailableCategory::Object,
            &error,
        )),
    }
}

fn ensure_contained_regular_file(path: &Path, root: &Path) -> Result<(), CefPreflightError> {
    if !path.starts_with(root) {
        return Err(invalid_runtime());
    }
    let metadata = path
        .symlink_metadata()
        .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))?;
    metadata
        .file_type()
        .is_file()
        .then_some(())
        .ok_or_else(invalid_runtime)
}

fn invalid_runtime() -> CefPreflightError {
    CefPreflightError::deterministic(CefUnavailableCategory::Object)
}
