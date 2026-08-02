use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const MAX_RESOURCE_DIRS: usize =
    crate::services::agent_import::MAX_ENABLED_RESOURCE_DIRS + 2;
pub const MAX_RESOURCE_FILES: usize =
    crate::services::agent_import::MAX_ENABLED_RESOURCE_FILES + 3;

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
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();
    for path in [data_dir.join("memory"), data_dir.join("skills")]
        .into_iter()
        .chain(imported.directories)
    {
        if directories.len() >= MAX_RESOURCE_DIRS {
            break;
        }
        if let Some(path) = canonical_local(&path, true) {
            directories.insert(path);
        }
    }
    let instruction_files = std::iter::once("AGENTS.md".to_string()).chain(
        crate::services::agent_import::enabled_hidden_documents(data_dir),
    );
    for path in instruction_files
        .map(|name| data_dir.join(name))
        .chain(imported.files)
    {
        if files.len() >= MAX_RESOURCE_FILES {
            break;
        }
        if let Some(path) = canonical_local(&path, false) {
            files.insert(path);
        }
    }
    AgentResourceAccess {
        directories: directories.into_iter().collect(),
        files: files.into_iter().collect(),
    }
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn canonical_local(path: &Path, directory: bool) -> Option<PathBuf> {
    if is_symlink(path) {
        return None;
    }
    let canonical = dunce::canonicalize(path).ok()?;
    let stable = !is_symlink(path) && dunce::canonicalize(path).ok().as_ref() == Some(&canonical);
    let expected_kind = if directory {
        canonical.is_dir()
    } else {
        canonical.is_file()
    };
    (stable && expected_kind).then_some(canonical)
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
}
