use super::limits::{
    MAX_INSTRUCTION_BYTES, MAX_RULES_PER_SOURCE, MAX_SCAN_DEPTH, MAX_SCAN_ENTRIES,
};
use super::walker::WalkResult;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

const EXCLUDED_DOCS: &[&str] = &["SOUL.md", "MEMORY.md", "IDENTITY.md", "USER.md", "TOOLS.md"];

pub fn find_rules(roots: &[PathBuf], _home: &Path) -> WalkResult {
    let mut result = WalkResult::default();
    let allowed_roots = canonical_roots(roots);
    let mut queue = roots
        .iter()
        .filter(|root| root.is_dir())
        .map(|root| (root.clone(), 0_usize))
        .collect::<VecDeque<_>>();
    for root in roots.iter().filter(|root| root.is_file()) {
        add_rule(root, &allowed_roots, &mut result);
    }
    let mut visited = 0_usize;
    while let Some((directory, depth)) = queue.pop_front() {
        if visited >= MAX_SCAN_ENTRIES || result.files.len() >= MAX_RULES_PER_SOURCE {
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
        if super::walker::excluded_dir(&directory) || super::walker::excluded_dir(&canonical) {
            continue;
        }
        if !is_allowed(&canonical, &allowed_roots) {
            result.had_error = true;
            continue;
        }
        let Ok(read_dir) = std::fs::read_dir(&directory) else {
            result.had_error = true;
            continue;
        };
        let remaining = MAX_SCAN_ENTRIES.saturating_sub(visited);
        let (paths, truncated, read_error) = super::walker::bounded_paths(read_dir, remaining);
        result.partial |= truncated;
        result.had_error |= read_error;
        visited += paths.len();
        for path in paths {
            if path.is_dir() {
                queue.push_back((path, depth + 1));
            } else {
                add_rule(&path, &allowed_roots, &mut result);
            }
        }
    }
    result.files.sort();
    result.files.dedup();
    result.files.truncate(MAX_RULES_PER_SOURCE);
    result
}

fn add_rule(path: &Path, roots: &[PathBuf], result: &mut WalkResult) {
    match rule_file(path, roots) {
        FileMatch::Found => result.files.push(path.to_path_buf()),
        FileMatch::Rejected => result.partial = true,
        FileMatch::None => {}
    }
}

fn rule_file(path: &Path, roots: &[PathBuf]) -> FileMatch {
    if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("md") {
        return FileMatch::None;
    }
    let excluded = path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            EXCLUDED_DOCS
                .iter()
                .any(|excluded| name.eq_ignore_ascii_case(excluded))
        });
    if excluded {
        return FileMatch::None;
    }
    let Ok(canonical) = path.canonicalize() else {
        return FileMatch::Rejected;
    };
    let small = std::fs::metadata(path)
        .map(|metadata| metadata.len() <= MAX_INSTRUCTION_BYTES)
        .unwrap_or(false);
    if is_allowed(&canonical, roots) && small {
        FileMatch::Found
    } else {
        FileMatch::Rejected
    }
}

fn canonical_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect()
}

fn is_allowed(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

enum FileMatch {
    Found,
    Rejected,
    None,
}
