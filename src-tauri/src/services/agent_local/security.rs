use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const DESTRUCTIVE_PATTERNS: &[&str] = &[
    "sudo rm",
    "chmod 777",
    "dd if=",
    "mkfs.",
    "> /dev/sd",
    "fdisk",
    "shutdown",
    "reboot",
    "init 0",
    "init 6",
    ":(){:|:&};:",
    "del /f /s /q",
    "rd /s /q",
    "format c:",
    "format d:",
];

static S7_EVAL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"eval\s+"?\$"#).unwrap());
static FIND_DELETE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bfind\b.*\s-delete\b").unwrap());
static RSYNC_DELETE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\brsync\b.*\s--delete\b").unwrap());
static DD_DEVICE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bdd\b.*\bof=/dev/").unwrap());

pub fn allowed_write_roots() -> Vec<PathBuf> {
    let mut roots = base_allowed_roots();
    append_agent_resources(&mut roots);
    roots
}

pub fn check_destructive_command(cmd: &str) -> Result<(), String> {
    let normalized = cmd.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_lower = normalized.to_ascii_lowercase();
    for pattern in DESTRUCTIVE_PATTERNS {
        if normalized_lower.contains(&pattern.to_ascii_lowercase()) {
            return Err(format!(
                "Commande bloquée : pattern dangereux « {pattern} »"
            ));
        }
    }
    if S7_EVAL_REGEX.is_match(&normalized)
        || FIND_DELETE_REGEX.is_match(&normalized)
        || RSYNC_DELETE_REGEX.is_match(&normalized)
        || DD_DEVICE_REGEX.is_match(&normalized)
        || normalized_lower.contains("mkfs ")
    {
        return Err("Commande bloquée : pattern dangereux détecté".into());
    }
    Ok(())
}

pub fn allowed_read_roots() -> Vec<PathBuf> {
    let mut roots = base_allowed_roots();
    if let Some(home) = dirs::home_dir() {
        for root in crate::services::agent_import::selected_skill_roots(&home) {
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
    }
    append_agent_resources(&mut roots);
    roots
}

fn base_allowed_roots() -> Vec<PathBuf> {
    let mut roots = super::directory_access::configured_roots().unwrap_or_default();
    append_configured_outputs_root(
        &mut roots,
        crate::services::config::session_outputs_directory(),
    );
    roots.push(crate::services::paths::data_dir());
    roots.push(std::env::temp_dir());
    roots
        .into_iter()
        .map(|path| path.canonicalize().unwrap_or(path))
        .collect()
}

fn append_agent_resources(roots: &mut Vec<PathBuf>) {
    let resources = super::agent_resource_access::current();
    for path in resources.directories.into_iter().chain(resources.files) {
        if !roots.contains(&path) {
            roots.push(path);
        }
    }
}

fn append_configured_outputs_root(roots: &mut Vec<PathBuf>, output_root: Option<PathBuf>) {
    if let Some(output_root) = output_root {
        roots.push(output_root);
    }
}

pub fn validate_read_path(path: &Path, working_dir: &Path) -> Result<PathBuf, String> {
    let canonical = canonicalize_candidate(path, working_dir)?;

    let working_canonical = working_dir
        .canonicalize()
        .unwrap_or_else(|_| working_dir.to_path_buf());
    if super::directory_access::ensure_allowed(&working_canonical).is_ok()
        && canonical.starts_with(&working_canonical)
    {
        return Ok(canonical);
    }

    let roots = allowed_read_roots();
    if roots.iter().any(|r| canonical.starts_with(r)) {
        Ok(canonical)
    } else {
        Err("Lecture interdite hors des zones autorisées".into())
    }
}

pub(crate) fn validate_read_path_in_roots(
    path: &Path,
    resolution_base: &Path,
    roots: &[PathBuf],
) -> Result<PathBuf, String> {
    let canonical = canonicalize_candidate(path, resolution_base)?;
    if roots.iter().any(|root| canonical.starts_with(root)) {
        Ok(canonical)
    } else {
        Err("Lecture interdite hors des zones autorisées".into())
    }
}

pub(crate) fn canonicalize_candidate(
    path: &Path,
    resolution_base: &Path,
) -> Result<PathBuf, String> {
    if path.exists() {
        return path.canonicalize().map_err(sanitize_error);
    }
    let parent = path.parent().ok_or("Chemin invalide")?;
    let filename = path.file_name().ok_or("Chemin sans nom de fichier")?;
    let canonical_parent = if parent.as_os_str().is_empty() {
        resolution_base.canonicalize().map_err(sanitize_error)?
    } else {
        parent.canonicalize().map_err(sanitize_error)?
    };
    Ok(canonical_parent.join(filename))
}

pub fn validate_write_path(path: &Path) -> Result<PathBuf, String> {
    let canonical = if path.exists() {
        path.canonicalize().map_err(sanitize_error)?
    } else {
        let parent = path.parent().ok_or("Chemin invalide")?;
        let filename = path.file_name().ok_or("Chemin sans nom de fichier")?;
        let canonical_parent = if parent.as_os_str().is_empty() {
            std::env::current_dir().map_err(sanitize_error)?
        } else {
            parent.canonicalize().map_err(sanitize_error)?
        };
        canonical_parent.join(filename)
    };

    let roots = allowed_write_roots();
    if roots.iter().any(|r| canonical.starts_with(r)) {
        Ok(canonical)
    } else {
        Err("Écriture interdite hors des zones autorisées (data, .ollama, temp, Projects)".into())
    }
}

pub fn sanitize_error<E: std::fmt::Display>(err: E) -> String {
    let msg = err.to_string();
    if msg.contains("No such file") || msg.contains("not found") {
        "Fichier introuvable".into()
    } else if msg.contains("Permission denied") {
        "Permission refusée".into()
    } else if msg.contains("Is a directory") {
        "Le chemin est un dossier".into()
    } else if msg.contains("Not a directory") {
        "Le chemin n'est pas un dossier".into()
    } else {
        "Erreur système".into()
    }
}

#[path = "security_tests.rs"]
#[cfg(test)]
mod tests;

#[path = "security_negative_tests.rs"]
#[cfg(test)]
mod negative_tests;

#[path = "security_output_roots_tests.rs"]
#[cfg(test)]
mod output_roots_tests;
