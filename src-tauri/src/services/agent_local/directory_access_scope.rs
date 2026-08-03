use std::path::{Path, PathBuf};

const ACCESS_ERROR: &str = "Accès au dossier refusé par les réglages.";

pub(crate) fn workspace_roots(working_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut roots = super::directory_access::configured_roots()?;
    let candidate = super::directory_access::canonical_access_path(working_dir)?;
    let managed = super::session_workspace::access_roots_for(&candidate);
    if !super::directory_access::is_path_in_roots(&candidate, &roots) && managed.is_empty() {
        return Err(ACCESS_ERROR.to_string());
    }
    append_unique(&mut roots, managed);
    if let Some(output) = crate::services::config::session_outputs_directory()
        .and_then(|path| dunce::canonicalize(path).ok())
        .filter(|path| path.is_dir())
    {
        append_unique(&mut roots, [output]);
    }
    if roots.len() > super::directory_access::MAX_WORKSPACE_ROOTS {
        return Err(ACCESS_ERROR.to_string());
    }
    Ok(roots)
}

pub(crate) fn roots_allow_full_disk(roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| unrestricted_root(root))
}

#[cfg(not(windows))]
fn unrestricted_root(root: &Path) -> bool {
    root.parent().is_none()
}

#[cfg(windows)]
fn unrestricted_root(root: &Path) -> bool {
    let expected = crate::models::config::default_allowed_paths()
        .into_iter()
        .next()
        .and_then(|path| dunce::canonicalize(path).ok());
    expected.is_some_and(|expected| {
        root.to_string_lossy()
            .eq_ignore_ascii_case(expected.to_string_lossy().as_ref())
    })
}

fn append_unique(roots: &mut Vec<PathBuf>, additions: impl IntoIterator<Item = PathBuf>) {
    for path in additions {
        if !roots.contains(&path) {
            roots.push(path);
        }
    }
}
