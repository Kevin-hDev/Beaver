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
    paths.sort_by_key(|path| path.file_name().map(|name| name != main.as_str()));
    Ok(paths)
}

pub(super) fn remove_all_in(dir: &Path, id: &str) -> Result<(), String> {
    let paths = paths_in(dir, id)?;
    let mut found_main = false;
    for path in paths {
        if path.file_name().and_then(|name| name.to_str()) == Some(&format!("{id}.json")) {
            found_main = true;
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
}
