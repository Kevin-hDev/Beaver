use super::limits::{MAX_MANIFEST_BYTES, MAX_SCAN_DEPTH, MAX_SCAN_ENTRIES, MAX_SKILLS_PER_SOURCE};
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

const SKILL_FILES: &[&str] = &["SKILL.md", "skill.md"];
const EXCLUDED_DIRS: &[&str] = &[
    ".system",
    "node_modules",
    "history",
    "histories",
    "sessions",
    "logs",
    "cache",
    "caches",
    "credentials",
];
#[derive(Default)]
pub struct WalkResult {
    pub files: Vec<PathBuf>,
    pub had_error: bool,
    pub partial: bool,
}

pub fn find_skills(roots: &[PathBuf], allowed_home: &Path) -> WalkResult {
    walk(roots, allowed_home, MAX_SKILLS_PER_SOURCE, |path| {
        for name in SKILL_FILES {
            let candidate = path.join(name);
            if candidate.is_file() {
                return if is_small_file(&candidate, MAX_MANIFEST_BYTES) {
                    DirectoryMatch::Found(candidate)
                } else {
                    DirectoryMatch::Rejected
                };
            }
        }
        DirectoryMatch::None
    })
}

fn walk<F>(roots: &[PathBuf], allowed_home: &Path, limit: usize, mut match_dir: F) -> WalkResult
where
    F: FnMut(&Path) -> DirectoryMatch,
{
    let allowed_roots = canonical_roots(roots, allowed_home);
    let mut queue = roots
        .iter()
        .filter(|root| root.is_dir())
        .map(|root| (root.clone(), 0_usize))
        .collect::<VecDeque<_>>();
    let mut result = WalkResult::default();
    let mut visited = 0_usize;

    while let Some((directory, depth)) = queue.pop_front() {
        if visited >= MAX_SCAN_ENTRIES || result.files.len() >= limit {
            result.partial = true;
            break;
        }
        if depth > MAX_SCAN_DEPTH {
            result.partial = true;
            continue;
        }
        let Ok(canonical) = directory.canonicalize() else {
            result.had_error = true;
            continue;
        };
        if excluded_dir(&directory) || excluded_dir(&canonical) {
            continue;
        }
        if !is_allowed(&canonical, &allowed_roots) {
            result.had_error = true;
            continue;
        }
        match match_dir(&directory) {
            DirectoryMatch::Found(found) => {
                result.files.push(found);
                continue;
            }
            DirectoryMatch::Rejected => {
                result.partial = true;
                continue;
            }
            DirectoryMatch::None => {}
        }
        let Ok(read_dir) = std::fs::read_dir(&directory) else {
            result.had_error = true;
            continue;
        };
        let remaining = MAX_SCAN_ENTRIES.saturating_sub(visited);
        let (children, truncated, read_error) = bounded_paths(read_dir, remaining);
        result.partial |= truncated;
        result.had_error |= read_error;
        visited += children.len();
        for child in children.into_iter().filter(|path| path.is_dir()) {
            queue.push_back((child, depth + 1));
        }
    }
    result
}

fn canonical_roots(roots: &[PathBuf], home: &Path) -> Vec<PathBuf> {
    let _ = home;
    let mut allowed = Vec::with_capacity(roots.len());
    for root in roots {
        if let Ok(root) = root.canonicalize() {
            if !allowed.contains(&root) {
                allowed.push(root);
            }
        }
    }
    allowed
}

fn is_allowed(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

pub(super) fn excluded_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| EXCLUDED_DIRS.contains(&name))
}

pub(super) fn bounded_paths(
    read_dir: std::fs::ReadDir,
    limit: usize,
) -> (Vec<PathBuf>, bool, bool) {
    let mut paths = BTreeMap::new();
    let mut truncated = false;
    let mut had_error = false;
    for entry in read_dir {
        match entry {
            Ok(entry) => {
                paths.insert(entry.file_name(), entry.path());
                if paths.len() > limit {
                    paths.pop_last();
                    truncated = true;
                }
            }
            Err(_) => had_error = true,
        }
    }
    (paths.into_values().collect(), truncated, had_error)
}

fn is_small_file(path: &Path, max_bytes: u64) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.len() <= max_bytes)
        .unwrap_or(false)
}

enum DirectoryMatch {
    Found(PathBuf),
    Rejected,
    None,
}
