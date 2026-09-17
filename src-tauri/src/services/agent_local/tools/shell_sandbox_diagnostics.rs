use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "sandbox-diagnostic.json";
const MAX_BYTES: u64 = 256;

#[derive(Deserialize, Serialize)]
struct Diagnostic {
    path_limit: bool,
    read_limit: bool,
    write_limit: bool,
    cache_unavailable: bool,
    isolation_unavailable: bool,
}

pub(super) fn record(
    temp_dir: &Path,
    path_limit: bool,
    read_limit: bool,
    write_limit: bool,
    cache_unavailable: bool,
    isolation_unavailable: bool,
) {
    if !path_limit
        && !read_limit
        && !write_limit
        && !cache_unavailable
        && !isolation_unavailable
    {
        return;
    }
    let value = Diagnostic {
        path_limit,
        read_limit,
        write_limit,
        cache_unavailable,
        isolation_unavailable,
    };
    let Ok(bytes) = serde_json::to_vec(&value) else {
        return;
    };
    if bytes.len() < MAX_BYTES as usize {
        let _ = crate::services::private_store::atomic_write(&path(temp_dir), &bytes);
    }
}

pub(super) fn warning(temp_dir: &Path) -> Option<String> {
    let path = path(temp_dir);
    let metadata = path.symlink_metadata().ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() >= MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() as u64 != metadata.len() || bytes.len() >= MAX_BYTES as usize {
        return None;
    }
    let value: Diagnostic = serde_json::from_slice(&bytes).ok()?;
    let mut warnings = Vec::with_capacity(2);
    if value.path_limit || value.read_limit || value.write_limit {
        warnings.push(
            "La limite de dossiers a écarté des racines d’outils excédentaires ; certains outils peuvent être indisponibles pour cette commande.",
        );
    }
    if value.cache_unavailable {
        warnings.push(
            "Un cache d’outil requis n’a pas pu être préparé ; l’outil concerné peut être indisponible pour cette commande.",
        );
    }
    if value.isolation_unavailable {
        #[cfg(target_os = "linux")]
        warnings.push(
            "Le noyau Linux doit être en version 6.2 ou plus récente pour limiter les dossiers du shell ; la commande a été bloquée sans élargir ses accès.",
        );
        #[cfg(not(target_os = "linux"))]
        warnings.push(
            "Le système ne peut pas appliquer la limite de dossiers à cette commande ; elle a été bloquée sans élargir ses accès.",
        );
    }
    (!warnings.is_empty()).then(|| warnings.join(" "))
}

fn path(temp_dir: &Path) -> PathBuf {
    temp_dir.join(FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_diagnostic_is_private_bounded_and_path_free() {
        let temp = tempfile::tempdir().expect("tempdir");
        record(temp.path(), true, true, true, true, true);

        let diagnostic = std::fs::read(path(temp.path())).expect("diagnostic");
        let warning = warning(temp.path()).expect("warning");

        assert!(diagnostic.len() < MAX_BYTES as usize);
        assert!(!String::from_utf8_lossy(&diagnostic).contains(temp.path().to_string_lossy().as_ref()));
        assert!(warning.contains("racines d’outils"));
        assert!(warning.contains("cache d’outil"));
        assert!(warning.contains("bloquée sans élargir"));
    }
}
