use sha2::{Digest, Sha256};
use std::path::Path;

const HASH_SEPARATOR: &str = "--";
const SHORT_HASH_LEN: usize = 8;
const LEGACY_HASH_LEN: usize = 24;
pub(crate) const MAX_PROJECT_FOLDER_ID_BYTES: usize = 180;

pub(crate) struct ProjectIdentity {
    pub id: String,
    pub label: String,
    pub legacy_id: String,
}

pub(crate) fn project_identity(working_dir: &Path) -> Result<ProjectIdentity, String> {
    let canonical = working_dir
        .canonicalize()
        .map_err(|error| storage_error("project canonicalize", error))?;
    let path = canonical.to_string_lossy();
    let digest = hex::encode(Sha256::digest(path.as_bytes()));
    let suffix = &digest[..SHORT_HASH_LEN];
    let slug_budget =
        MAX_PROJECT_FOLDER_ID_BYTES.saturating_sub(HASH_SEPARATOR.len() + SHORT_HASH_LEN);
    let slug = truncate_slug(&readable_slug(&path), slug_budget);
    let label = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Projet")
        .chars()
        .take(80)
        .collect();
    Ok(ProjectIdentity {
        id: format!("{slug}{HASH_SEPARATOR}{suffix}"),
        label,
        legacy_id: digest[..LEGACY_HASH_LEN].to_string(),
    })
}

pub fn valid_project_id(id: &str) -> bool {
    if valid_legacy_id(id) {
        return true;
    }
    if id.is_empty() || id.len() > MAX_PROJECT_FOLDER_ID_BYTES {
        return false;
    }
    let Some((slug, suffix)) = id.rsplit_once(HASH_SEPARATOR) else {
        return false;
    };
    !slug.is_empty()
        && slug.chars().any(char::is_alphanumeric)
        && slug
            .chars()
            .all(|value| value.is_alphanumeric() || value == '-')
        && suffix.len() == SHORT_HASH_LEN
        && suffix
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
}

fn valid_legacy_id(id: &str) -> bool {
    id.len() == LEGACY_HASH_LEN && id.bytes().all(|value| value.is_ascii_hexdigit())
}

fn readable_slug(path: &str) -> String {
    let slug = path
        .chars()
        .map(|value| {
            if value.is_alphanumeric() || value == '-' {
                value
            } else {
                '-'
            }
        })
        .collect::<String>();
    if slug.chars().any(char::is_alphanumeric) {
        slug
    } else {
        "project".to_string()
    }
}

fn truncate_slug(slug: &str, max_bytes: usize) -> String {
    if slug.len() <= max_bytes {
        return slug.to_string();
    }
    let tail_budget = max_bytes.saturating_sub(1);
    let mut start = slug.len().saturating_sub(tail_budget);
    while !slug.is_char_boundary(start) {
        start += 1;
    }
    let tail = slug[start..].trim_start_matches('-');
    if tail.is_empty() {
        "project".to_string()
    } else {
        format!("-{tail}")
    }
}

fn storage_error(operation: &str, error: std::io::Error) -> String {
    ::log::error!("[memory] {operation}: {error}");
    "Mémoire indisponible.".to_string()
}

#[cfg(test)]
#[path = "memory_project_id_tests.rs"]
mod tests;
