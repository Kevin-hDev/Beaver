use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryArea {
    Global,
    Projects,
}

#[derive(Debug)]
pub struct ClassifiedMemoryPath {
    pub area: MemoryArea,
    area_views: Vec<PathBuf>,
}

impl ClassifiedMemoryPath {
    pub fn belongs_exclusively_to(&self, scope_root: &Path) -> bool {
        let roots = path_views(scope_root);
        self.area_views
            .iter()
            .all(|candidate| roots.iter().any(|root| candidate.starts_with(root)))
    }
}

pub fn classify_memory_path(
    raw_path: &str,
    working_dir: Option<&Path>,
    memory_root: &Path,
) -> Result<Option<ClassifiedMemoryPath>, String> {
    if !Path::new(raw_path).is_absolute() && working_dir.is_none() {
        return Ok(None);
    }
    let candidates = candidate_views(raw_path, working_dir)?;
    let roots = path_views(memory_root);
    let mut global_views = Vec::new();
    let mut project_views = Vec::new();

    for candidate in candidates {
        let mut is_global = false;
        let mut is_project = false;
        for root in &roots {
            is_global |= candidate.starts_with(root.join("global"));
            is_project |= candidate.starts_with(root.join("projects"));
        }
        if is_global {
            push_unique(&mut global_views, candidate.clone());
        }
        if is_project {
            push_unique(&mut project_views, candidate);
        }
    }

    match (global_views.is_empty(), project_views.is_empty()) {
        (true, true) => Ok(None),
        (false, true) => Ok(Some(ClassifiedMemoryPath {
            area: MemoryArea::Global,
            area_views: global_views,
        })),
        (true, false) => Ok(Some(ClassifiedMemoryPath {
            area: MemoryArea::Projects,
            area_views: project_views,
        })),
        (false, false) => Err("Chemin mémoire interdit.".into()),
    }
}

fn candidate_views(
    raw_path: &str,
    working_dir: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    if raw_path.is_empty() || raw_path.len() > 4_096 || raw_path.contains('\0') {
        return Err("Chemin mémoire invalide.".into());
    }
    let path = Path::new(raw_path);
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir
            .ok_or_else(|| "Chemin mémoire invalide.".to_string())?
            .join(path)
    };
    Ok(path_views(&candidate))
}

fn path_views(path: &Path) -> Vec<PathBuf> {
    let lexical = normalize(path);
    let mut views = vec![lexical.clone()];
    if let Some(canonical) = canonicalize_existing_ancestor(&lexical) {
        push_unique(&mut views, canonical);
    }
    views
}

fn canonicalize_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    let mut missing = Vec::<OsString>::new();
    loop {
        match std::fs::canonicalize(current) {
            Ok(mut resolved) => {
                for component in missing.iter().rev() {
                    resolved.push(component);
                }
                return Some(resolved);
            }
            Err(_) => {
                missing.push(current.file_name()?.to_os_string());
                current = current.parent()?;
            }
        }
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(normalized.components().next_back(), Some(Component::Normal(_))) {
                    normalized.pop();
                } else if !path.is_absolute() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn push_unique(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.contains(&candidate) {
        paths.push(candidate);
    }
}
