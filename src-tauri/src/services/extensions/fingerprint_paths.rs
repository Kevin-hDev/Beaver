use std::path::{Component, Path};

pub(super) fn reject_symlink(path: &Path) -> Result<std::fs::Metadata, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| error())?;
    if metadata.file_type().is_symlink() {
        return Err(error());
    }
    Ok(metadata)
}

pub(super) fn clean_relative(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error());
    }
    Ok(path)
}

pub(super) fn clean_root(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    let mut has_normal_component = false;
    if !path.is_absolute() {
        return Err(error());
    }
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir | Component::ParentDir => return Err(error()),
        }
    }
    has_normal_component.then_some(path).ok_or_else(error)
}

pub(super) fn relative_name(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| error())?;
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str().ok_or_else(error),
            _ => Err(error()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    (!components.is_empty())
        .then(|| components.join("/"))
        .ok_or_else(error)
}

pub(super) fn excluded_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "node_modules"))
}

fn error() -> String {
    super::error_codes::FINGERPRINT_FAILED.to_string()
}
