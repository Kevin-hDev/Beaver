// The durable Ollama receipt stores process identity but no port, so this command reports exact
// process presence instead of claiming that the HTTP service responds.
use crate::app_detect::{app_is_running, ollama_process_running};
use crate::output::{format_size, Out};
use std::path::{Path, PathBuf};

const MAX_SCAN_DEPTH: usize = 64;
const MAX_ENTRIES_PER_DIRECTORY: usize = 100_000;

pub fn dir_size(path: &Path) -> u64 {
    dir_size_at(path, 0)
}

fn dir_size_at(path: &Path, depth: usize) -> u64 {
    if depth >= MAX_SCAN_DEPTH {
        return 0;
    }
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if metadata.file_type().is_symlink() {
        return 0;
    }
    if metadata.is_file() {
        return metadata.len();
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .take(MAX_ENTRIES_PER_DIRECTORY)
        .filter_map(Result::ok)
        .map(|entry| dir_size_at(&entry.path(), depth + 1))
        .fold(0_u64, u64::saturating_add)
}

fn regular_file_count(path: &Path) -> usize {
    regular_file_count_at(path, 0)
}

fn regular_file_count_at(path: &Path, depth: usize) -> usize {
    if depth >= MAX_SCAN_DEPTH {
        return 0;
    }
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if metadata.file_type().is_symlink() {
        return 0;
    }
    if metadata.is_file() {
        return 1;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .take(MAX_ENTRIES_PER_DIRECTORY)
        .filter_map(Result::ok)
        .map(|entry| regular_file_count_at(&entry.path(), depth + 1))
        .fold(0_usize, usize::saturating_add)
}

fn largest_directories(root: &Path) -> Vec<(PathBuf, u64)> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut largest = Vec::with_capacity(3);
    for entry in entries
        .take(MAX_ENTRIES_PER_DIRECTORY)
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        largest.push((path, dir_size(&entry.path())));
        largest.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        largest.truncate(3);
    }
    largest
}

pub fn run(out: &Out) -> i32 {
    let app = if app_is_running() {
        out.t("ouverte", "open")
    } else {
        out.t("fermée", "closed")
    };
    out.line(&format!("{} {app}", out.t("Application :", "Application:")));

    let ollama = if ollama_process_running() {
        out.t("présent", "present")
    } else {
        out.t("absent", "absent")
    };
    out.line(&format!(
        "{} {ollama}",
        out.t("Processus Ollama :", "Ollama process:")
    ));

    match cl_go_dash_lib::cli_support::ollama_models_dir() {
        Some(models) => {
            let count = regular_file_count(&models.join("manifests"));
            out.line(&format!(
                "{} {count}, {}",
                out.t("Modèles installés :", "Installed models:"),
                format_size(dir_size(&models))
            ));
        }
        None => out.line(out.t(
            "Modèles installés : emplacement indisponible",
            "Installed models: location unavailable",
        )),
    }

    let root = cl_go_dash_lib::cli_support::data_dir();
    out.line(&format!(
        "{} {}",
        out.t("Données :", "Data:"),
        format_size(dir_size(&root))
    ));
    for (path, bytes) in largest_directories(&root) {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("?");
        out.line(&format!("  {name}: {}", format_size(bytes)));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_size_compte_recursivement() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        std::fs::write(directory.path().join("a.txt"), vec![0_u8; 100]).expect("first file");
        std::fs::create_dir(directory.path().join("sub")).expect("subdirectory");
        std::fs::write(directory.path().join("sub/b.txt"), vec![0_u8; 50]).expect("second file");
        assert_eq!(dir_size(directory.path()), 150);
    }

    #[test]
    fn dir_size_dossier_absent_vaut_zero() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        assert_eq!(dir_size(&directory.path().join("missing")), 0);
    }

    #[cfg(unix)]
    #[test]
    fn dir_size_ne_suit_pas_un_lien_symbolique() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::TempDir::new().expect("temporary directory");
        let outside = tempfile::TempDir::new().expect("outside directory");
        std::fs::write(outside.path().join("secret"), vec![0_u8; 100]).expect("outside file");
        symlink(outside.path(), directory.path().join("linked")).expect("symlink");
        assert_eq!(dir_size(directory.path()), 0);
    }
}
