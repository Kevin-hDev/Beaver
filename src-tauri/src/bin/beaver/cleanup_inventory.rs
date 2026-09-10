use super::{Candidate, CandidateKind, InventoryError};
use std::path::Path;
use std::time::{Duration, SystemTime};

const MAX_LOG_AGE_DAYS: u64 = 30;
const MAX_FAMILY_ENTRIES: usize = 10_000;

fn inventory_error(family: &'static str) -> InventoryError {
    InventoryError { family }
}

fn family_entries(
    path: &Path,
    family: &'static str,
) -> Result<Option<std::fs::ReadDir>, InventoryError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(inventory_error(family)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(inventory_error(family));
    }
    std::fs::read_dir(path)
        .map(Some)
        .map_err(|_| inventory_error(family))
}

fn is_rotated_app_log(name: &str) -> bool {
    name.starts_with("beaver_") && (name.ends_with(".log") || name.contains(".log."))
}

fn collect_old_logs(
    root: &Path,
    now: SystemTime,
    candidates: &mut Vec<Candidate>,
) -> Result<(), InventoryError> {
    let logs = root.join("logs");
    let Some(entries) = family_entries(&logs, "logs")? else {
        return Ok(());
    };
    let cutoff = now
        .checked_sub(Duration::from_secs(MAX_LOG_AGE_DAYS * 86_400))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    for (index, entry) in entries.enumerate() {
        if index == MAX_FAMILY_ENTRIES {
            return Err(inventory_error("logs"));
        }
        let entry = entry.map_err(|_| inventory_error("logs"))?;
        let metadata =
            std::fs::symlink_metadata(entry.path()).map_err(|_| inventory_error("logs"))?;
        let name = entry.file_name();
        let old = metadata.modified().map_err(|_| inventory_error("logs"))? < cutoff;
        if metadata.is_file()
            && !metadata.file_type().is_symlink()
            && name.to_str().is_some_and(is_rotated_app_log)
            && old
        {
            candidates.push(Candidate {
                path: entry.path(),
                bytes: metadata.len(),
                reason_fr: "rotation de journal de plus de 30 jours",
                reason_en: "log rotation older than 30 days",
                family: logs.clone(),
                kind: CandidateKind::File,
            });
        }
    }
    Ok(())
}

fn collect_bundle_temporaries(
    root: &Path,
    candidates: &mut Vec<Candidate>,
) -> Result<(), InventoryError> {
    let bundle = cl_go_dash_lib::cli_support::ollama_bundle_dir(root);
    let receipt = cl_go_dash_lib::cli_support::ollama_bundle_receipt_tmp_path(root);
    let Some(entries) = family_entries(&bundle, "bundle Ollama")? else {
        return Ok(());
    };
    for (index, entry) in entries.enumerate() {
        if index == MAX_FAMILY_ENTRIES {
            return Err(inventory_error("bundle Ollama"));
        }
        let entry = entry.map_err(|_| inventory_error("bundle Ollama"))?;
        let metadata = std::fs::symlink_metadata(entry.path())
            .map_err(|_| inventory_error("bundle Ollama"))?;
        let temporary = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.ends_with(".tmp") || name.ends_with(".partial"));
        if (entry.path() == receipt || temporary)
            && metadata.is_file()
            && !metadata.file_type().is_symlink()
        {
            candidates.push(Candidate {
                path: entry.path(),
                bytes: metadata.len(),
                reason_fr: "téléchargement Ollama interrompu",
                reason_en: "interrupted Ollama download",
                family: bundle.clone(),
                kind: CandidateKind::File,
            });
        }
    }
    Ok(())
}

pub fn inventory(root: &Path, now: SystemTime) -> Result<Vec<Candidate>, InventoryError> {
    let mut candidates = Vec::new();
    collect_old_logs(root, now, &mut candidates)?;
    collect_bundle_temporaries(root, &mut candidates)?;
    for path in cl_go_dash_lib::cli_support::abandoned_ollama_staging_dirs(root)
        .map_err(|_| inventory_error("préparations Ollama"))?
    {
        candidates.push(Candidate {
            bytes: crate::status::dir_size(&path),
            path,
            reason_fr: "préparation Ollama abandonnée",
            reason_en: "abandoned Ollama staging",
            family: root.to_path_buf(),
            kind: CandidateKind::OllamaStaging,
        });
    }
    for (path, bytes) in cl_go_dash_lib::cli_support::old_tool_results(root, now)
        .map_err(|_| inventory_error("résultats d’outils"))?
    {
        candidates.push(Candidate {
            path,
            bytes,
            reason_fr: "résultats d’outils de plus de 24 h",
            reason_en: "tool results older than 24 hours",
            family: root.join("tool-results"),
            kind: CandidateKind::ToolResult,
        });
    }
    Ok(candidates)
}

pub fn is_safely_inside(family_dir: &Path, candidate: &Path) -> bool {
    let family_metadata = match std::fs::symlink_metadata(family_dir) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    let candidate_metadata = match std::fs::symlink_metadata(candidate) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    if family_metadata.file_type().is_symlink()
        || !family_metadata.is_dir()
        || candidate_metadata.file_type().is_symlink()
    {
        return false;
    }
    let Some(parent) = candidate.parent() else {
        return false;
    };
    family_dir
        .canonicalize()
        .ok()
        .zip(parent.canonicalize().ok())
        .is_some_and(|(family, parent)| family == parent)
}
