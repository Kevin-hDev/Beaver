use super::{result, CheckResult};
use std::io::{BufRead, Write};
use std::path::Path;
use sysinfo::Disks;

const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;
const MIN_FREE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ROOT_ENTRIES: usize = 10_000;

fn check_data_directory(root: &Path) -> CheckResult {
    let writable = std::fs::symlink_metadata(root)
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
        && tempfile::Builder::new()
            .prefix("doctor-write-probe-")
            .tempfile_in(root)
            .and_then(|mut file| file.write_all(b"beaver"))
            .is_ok();
    result(
        "Dossier de données",
        "Data directory",
        writable,
        "Vérifiez les droits d’écriture du dossier de données.",
        "Check the data directory write permissions.",
    )
}

fn read_small_regular(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_JSON_BYTES {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
    std::fs::read(path).map(Some)
}

fn check_config(root: &Path) -> CheckResult {
    let passed = match read_small_regular(&root.join("config.json")) {
        Ok(None) => true,
        Ok(Some(bytes)) => serde_json::from_slice::<serde_json::Value>(&bytes).is_ok(),
        Err(_) => false,
    };
    result(
        "Configuration config.json",
        "config file",
        passed,
        "Supprimez ou corrigez config.json ; Beaver repartira sur les réglages par défaut.",
        "Delete or fix config.json; Beaver will restart with defaults.",
    )
}

fn check_vault(root: &Path) -> CheckResult {
    let vault = root.join("secrets.enc");
    let vault_present = std::fs::symlink_metadata(vault)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
    let registry_has_data = std::fs::symlink_metadata(root.join("configured-providers.json"))
        .is_ok_and(|metadata| {
            metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
        });
    result(
        "Coffre de clés API",
        "API key vault",
        vault_present || !registry_has_data,
        "Reconnectez les fournisseurs configurés pour recréer le coffre.",
        "Reconnect configured providers to recreate the vault.",
    )
}

fn check_interrupted_download(root: &Path) -> CheckResult {
    let temporary = cl_go_dash_lib::cli_support::ollama_bundle_receipt_tmp_path(root);
    let passed = std::fs::symlink_metadata(temporary)
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    result(
        "Téléchargement Ollama interrompu",
        "Interrupted Ollama download",
        passed,
        "Relancez Beaver pour reprendre, ou utilisez beaver cleanup.",
        "Restart Beaver to resume, or use beaver cleanup.",
    )
}

fn check_bundle(root: &Path) -> CheckResult {
    let bundle = cl_go_dash_lib::cli_support::ollama_bundle_dir(root);
    let absent = std::fs::symlink_metadata(&bundle)
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    let passed = absent || cl_go_dash_lib::cli_support::ollama_bundle_is_valid(root);
    result(
        "Bundle Ollama",
        "Ollama bundle",
        passed,
        "Relancez Beaver pour retélécharger le bundle incomplet.",
        "Restart Beaver to download the incomplete bundle again.",
    )
}

fn check_staging(root: &Path) -> CheckResult {
    let passed = cl_go_dash_lib::cli_support::abandoned_ollama_staging_dirs(root)
        .is_ok_and(|directories| directories.is_empty());
    result(
        "Préparations Ollama",
        "Ollama staging",
        passed,
        "Utilisez beaver cleanup pour retirer les préparations abandonnées.",
        "Use beaver cleanup to remove abandoned staging directories.",
    )
}

fn line_limit_ok(path: &Path, max_lines: usize, max_bytes: u64) -> bool {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max_bytes {
        return false;
    }
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut reader = std::io::BufReader::new(file);
    let mut lines = 0_usize;
    let mut trailing_line = false;
    loop {
        let Ok(buffer) = reader.fill_buf() else {
            return false;
        };
        if buffer.is_empty() {
            return lines.saturating_add(usize::from(trailing_line)) <= max_lines;
        }
        lines = lines.saturating_add(buffer.iter().filter(|byte| **byte == b'\n').count());
        trailing_line = buffer.last() != Some(&b'\n');
        if lines > max_lines {
            return false;
        }
        let consumed = buffer.len();
        reader.consume(consumed);
    }
}

fn check_logs(root: &Path) -> CheckResult {
    let logs = root.join("logs");
    let app = line_limit_ok(
        &logs.join("beaver.log"),
        usize::MAX,
        cl_go_dash_lib::cli_support::APP_LOG_MAX_BYTES.saturating_mul(2),
    );
    let wakeup_lines = cl_go_dash_lib::cli_support::WAKEUP_LOG_MAX_LINES.saturating_mul(2);
    let wakeup_bytes =
        wakeup_lines.saturating_mul(cl_go_dash_lib::cli_support::WAKEUP_LOG_MAX_LINE_BYTES) as u64;
    let wakeups = line_limit_ok(&logs.join("wakeups.jsonl"), wakeup_lines, wakeup_bytes);
    result(
        "Taille des journaux actifs",
        "Active log limits",
        app && wakeups,
        "Redémarrez Beaver ; si le journal regrossit, signalez le problème.",
        "Restart Beaver; if the log grows again, report the problem.",
    )
}

fn check_root_temporaries(root: &Path) -> CheckResult {
    let passed = std::fs::read_dir(root).is_ok_and(|entries| {
        entries
            .take(MAX_ROOT_ENTRIES + 1)
            .enumerate()
            .all(|(index, entry)| {
                index < MAX_ROOT_ENTRIES
                    && entry
                        .is_ok_and(|entry| !entry.file_name().to_string_lossy().ends_with(".tmp"))
            })
    });
    result(
        "Fichiers temporaires",
        "Root temporary files",
        passed,
        "Fermez Beaver puis utilisez beaver cleanup.",
        "Close Beaver, then use beaver cleanup.",
    )
}

fn check_disk(root: &Path) -> CheckResult {
    let free = root.canonicalize().ok().and_then(|root| {
        Disks::new_with_refreshed_list()
            .list()
            .iter()
            .filter(|disk| root.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().components().count())
            .map(|disk| disk.available_space())
    });
    result(
        "Espace disque disponible",
        "Free disk space",
        free.is_some_and(|bytes| bytes >= MIN_FREE_BYTES),
        "Libérez au moins 2 Go d’espace disque.",
        "Free at least 2 GB of disk space.",
    )
}

pub(super) fn run_checks(root: &Path) -> Vec<CheckResult> {
    vec![
        check_data_directory(root),
        check_config(root),
        check_vault(root),
        check_interrupted_download(root),
        check_bundle(root),
        check_staging(root),
        check_logs(root),
        check_root_temporaries(root),
        check_disk(root),
    ]
}
