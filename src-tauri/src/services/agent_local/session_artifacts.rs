use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::session_limits::MAX_SESSION_FILES;

pub(super) fn paths_in(dir: &Path, id: &str) -> Result<Vec<PathBuf>, String> {
    super::session_store::validate_session_id(id)?;
    let main = format!("{id}.json");
    let backup = format!("{main}.v1.bak");
    let mut paths = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(paths),
        Err(_) => return Err(delete_failed()),
    };
    for (index, entry) in entries.enumerate() {
        if index >= MAX_SESSION_FILES {
            return Err(delete_failed());
        }
        let entry = entry.map_err(|_| delete_failed())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(delete_failed)?;
        if name == main || name == backup || is_known_temp(name, &main, &backup) {
            let file_type = entry.file_type().map_err(|_| delete_failed())?;
            if !file_type.is_file() && !file_type.is_symlink() {
                return Err(delete_failed());
            }
            paths.push(entry.path());
        }
    }
    paths.sort_by_key(|path| path.file_name().map(|name| name == main.as_str()));
    Ok(paths)
}

pub(super) fn remove_all_in(dir: &Path, id: &str) -> Result<(), String> {
    remove_all_in_inner(dir, id, false)
}

#[cfg(test)]
pub(super) fn remove_all_in_fail_before_main(dir: &Path, id: &str) -> Result<(), String> {
    remove_all_in_inner(dir, id, true)
}

fn remove_all_in_inner(dir: &Path, id: &str, fail_before_main: bool) -> Result<(), String> {
    let paths = paths_in(dir, id)?;
    let main = format!("{id}.json");
    let mut found_main = false;
    for path in paths {
        if path.file_name().and_then(|name| name.to_str()) == Some(main.as_str()) {
            found_main = true;
            if fail_before_main {
                return Err(delete_failed());
            }
        }
        std::fs::remove_file(path).map_err(|_| delete_failed())?;
    }
    if found_main {
        Ok(())
    } else {
        Err("Session introuvable".to_string())
    }
}

fn is_known_temp(name: &str, main: &str, backup: &str) -> bool {
    [main, backup].iter().any(|base| {
        let Some(random) = name
            .strip_prefix(&format!(".{base}."))
            .and_then(|suffix| suffix.strip_suffix(".tmp"))
        else {
            return false;
        };
        random.len() == 32 && random.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn delete_failed() -> String {
    "Suppression de session impossible".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_exact_private_store_temps_match() {
        assert!(super::is_known_temp(
            ".abc.json.0123456789abcdef0123456789abcdef.tmp",
            "abc.json",
            "abc.json.v1.bak"
        ));
        assert!(!super::is_known_temp(
            ".abc.json.short.tmp",
            "abc.json",
            "abc.json.v1.bak"
        ));
    }

    #[test]
    fn failure_after_artifacts_keeps_main_retryable_without_backup() {
        let root = tempfile::tempdir().unwrap();
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let main = root.path().join(format!("{id}.json"));
        let backup = root.path().join(format!("{id}.json.v1.bak"));
        let temp = root
            .path()
            .join(format!(".{id}.json.0123456789abcdef0123456789abcdef.tmp"));
        crate::services::private_store::atomic_write(&main, b"main").unwrap();
        crate::services::private_store::atomic_write(&backup, b"backup").unwrap();
        crate::services::private_store::atomic_write(&temp, b"temp").unwrap();

        assert!(super::remove_all_in_fail_before_main(root.path(), id).is_err());
        assert!(main.is_file());
        assert!(!backup.exists());
        assert!(!temp.exists());

        super::remove_all_in(root.path(), id).expect("retry deletion");
        assert!(!main.exists());
    }
}
