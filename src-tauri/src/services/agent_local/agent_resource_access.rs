use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const LOCAL_RESOURCE_DIRS: usize = 2;
pub const LOCAL_RESOURCE_FILES: usize = 3;
pub const MAX_RESOURCE_DIRS: usize =
    crate::services::agent_import::MAX_ENABLED_RESOURCE_DIRS + LOCAL_RESOURCE_DIRS;
pub const MAX_RESOURCE_FILES: usize =
    crate::services::agent_import::MAX_ENABLED_RESOURCE_FILES + LOCAL_RESOURCE_FILES;

#[derive(Default)]
pub struct AgentResourceAccess {
    pub directories: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

pub fn current() -> AgentResourceAccess {
    let data_dir = crate::services::paths::data_dir();
    let imported = dirs::home_dir()
        .map(|home| crate::services::agent_import::enabled_resource_paths(&home))
        .unwrap_or_default();
    collect(&data_dir, imported)
}

fn collect(
    data_dir: &Path,
    imported: crate::services::agent_import::EnabledResourcePaths,
) -> AgentResourceAccess {
    let data_root = stable_canonical(data_dir);
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();
    if let Some(data_root) = data_root.as_deref() {
        for path in [data_dir.join("memory"), data_dir.join("skills")] {
            if directories.len() >= MAX_RESOURCE_DIRS {
                break;
            }
            if let Some(path) = canonical_local(&path, data_root, true) {
                directories.insert(path);
            }
        }
    }
    append_imported(&mut directories, imported.directories, MAX_RESOURCE_DIRS, true);
    if let Some(data_root) = data_root.as_deref() {
        let instruction_files = std::iter::once("AGENTS.md".to_string()).chain(
            crate::services::agent_import::enabled_hidden_documents(data_dir),
        );
        for path in instruction_files.map(|name| data_dir.join(name)) {
            if files.len() >= MAX_RESOURCE_FILES {
                break;
            }
            if let Some(path) = canonical_local(&path, data_root, false) {
                files.insert(path);
            }
        }
    }
    append_imported(&mut files, imported.files, MAX_RESOURCE_FILES, false);
    AgentResourceAccess {
        directories: directories.into_iter().collect(),
        files: files.into_iter().collect(),
    }
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn canonical_local(path: &Path, data_root: &Path, directory: bool) -> Option<PathBuf> {
    let canonical = stable_canonical(path)?;
    (canonical.starts_with(data_root) && expected_kind(&canonical, directory)).then_some(canonical)
}

fn stable_canonical(path: &Path) -> Option<PathBuf> {
    if is_symlink(path) {
        return None;
    }
    let canonical = dunce::canonicalize(path).ok()?;
    let stable = !is_symlink(path) && dunce::canonicalize(path).ok().as_ref() == Some(&canonical);
    stable.then_some(canonical)
}

fn expected_kind(path: &Path, directory: bool) -> bool {
    if directory { path.is_dir() } else { path.is_file() }
}

fn append_imported(
    target: &mut BTreeSet<PathBuf>,
    paths: Vec<PathBuf>,
    limit: usize,
    directory: bool,
) {
    for path in paths {
        if target.len() >= limit {
            break;
        }
        if let Some(path) = stable_canonical(&path).filter(|path| expected_kind(path, directory)) {
            target.insert(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_access_exposes_resources_without_exposing_the_private_store() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data = temp.path().join("data");
        std::fs::create_dir_all(data.join("memory")).expect("memory");
        std::fs::create_dir_all(data.join("skills")).expect("skills");
        std::fs::write(data.join("AGENTS.md"), "rules").expect("instructions");

        let access = collect(&data, Default::default());
        let data = dunce::canonicalize(data).expect("data");

        assert_eq!(access.directories.len(), 2);
        assert!(access.directories.iter().all(|path| path.starts_with(&data)));
        assert!(access.directories.iter().all(|path| path != &data));
        assert_eq!(access.files, vec![data.join("AGENTS.md")]);
        assert!(canonical_local(temp.path(), &data, true).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn local_access_rejects_redirected_resource_roots() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("tempdir");
        let data = temp.path().join("data");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&data).expect("data");
        std::fs::create_dir_all(&outside).expect("outside");
        symlink(&outside, data.join("memory")).expect("memory symlink");

        let access = collect(&data, Default::default());

        assert!(access.directories.is_empty());
    }

    #[test]
    fn enabled_imported_resources_remain_available_outside_data_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data = temp.path().join("data");
        let imported = temp.path().join("external/rules");
        let document = temp.path().join("external/AGENTS.md");
        std::fs::create_dir_all(&data).expect("data");
        std::fs::create_dir_all(&imported).expect("imported rules");
        std::fs::write(&document, "rules").expect("imported document");

        let access = collect(
            &data,
            crate::services::agent_import::EnabledResourcePaths {
                directories: vec![imported.clone()],
                files: vec![document.clone()],
            },
        );

        assert!(access
            .directories
            .contains(&dunce::canonicalize(imported).expect("rules")));
        assert!(access
            .files
            .contains(&dunce::canonicalize(document).expect("document")));
    }
}
