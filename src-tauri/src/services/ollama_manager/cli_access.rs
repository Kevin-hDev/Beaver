pub(crate) fn models_directory_path() -> Option<std::path::PathBuf> {
    let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
    super::recovery_entry::frozen_models_directory(&paths)
        .map(|directory| directory.path().to_path_buf())
}

pub(crate) fn process_name_matches(name: &str) -> bool {
    if name == "ollama" || name.eq_ignore_ascii_case("ollama.exe") {
        return true;
    }
    #[cfg(unix)]
    return super::spawn_gate_unix::is_process_name(name);
    #[cfg(not(unix))]
    false
}

pub(crate) fn bundle_is_valid(root: &std::path::Path) -> bool {
    let paths = crate::services::paths::ollama_paths(root);
    matches!(
        super::startup_recovery::bundle_state(&paths),
        Ok(super::types::BundleState::Ready)
    )
}

pub(crate) fn bundle_receipt_tmp_path(root: &std::path::Path) -> std::path::PathBuf {
    let bundle = crate::services::paths::ollama_paths(root).active;
    crate::services::paths::bundle_receipt_tmp_path(&bundle)
}

pub(crate) fn abandoned_staging_dirs(
    root: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, super::error::OllamaErrorCode> {
    use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind};
    use super::recovery_decision::JournalPresence;

    let paths = crate::services::paths::ollama_paths(root);
    let fs = super::durable_fs::platform_fs();
    let journal =
        match fs.read_bounded(&paths.journal, super::constants::MAX_DURABLE_DOCUMENT_BYTES) {
            Ok(bytes) => super::journal::OllamaTransactionJournal::parse_bounded(&bytes)
                .map(JournalPresence::Valid)
                .map_err(|_| super::error::OllamaErrorCode::OllamaJournalInvalid)?,
            Err(error) if error.kind() == OllamaFsErrorKind::NotFound => JournalPresence::Absent,
            Err(_) => return Err(super::error::OllamaErrorCode::OllamaStorageUnavailable),
        };
    let snapshot = super::cleanup::snapshot(journal, &fs, &paths);
    let mut abandoned = Vec::with_capacity(2);
    match super::staging_recovery::decide(&snapshot.journal, &paths)? {
        Some(super::staging_recovery::StagingRecoveryAction::Move(source)) => {
            let path = match source {
                super::staging_recovery::StagingSource::Install => paths.install_staging.clone(),
                super::staging_recovery::StagingSource::Update => paths.update_staging.clone(),
                super::staging_recovery::StagingSource::Legacy => paths.legacy_staging.clone(),
            };
            abandoned.push(path);
        }
        Some(super::staging_recovery::StagingRecoveryAction::RemoveTrash) => {
            abandoned.push(paths.uncommitted_staging_delete.clone());
        }
        None => {}
    }
    match super::archive_recovery::decide(&snapshot)? {
        Some(super::archive_recovery::ArchiveRecoveryAction::MoveToFailed) => {
            abandoned.push(paths.archive_staging);
        }
        Some(super::archive_recovery::ArchiveRecoveryAction::RemoveFailed) => {
            abandoned.push(paths.archive_failed);
        }
        None => {}
    }
    Ok(abandoned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_orphelin_est_signale() {
        let root = tempfile::TempDir::new().expect("temporary directory");
        let staging = root.path().join("ollama-bundle-update-staging");
        std::fs::create_dir(&staging).expect("staging");
        assert_eq!(
            abandoned_staging_dirs(root.path()).expect("inventory"),
            vec![staging]
        );
    }

    #[test]
    fn staging_reference_par_une_reprise_est_preserve() {
        let root = tempfile::TempDir::new().expect("temporary directory");
        std::fs::create_dir(root.path().join("ollama-bundle-update-staging")).expect("staging");
        let digest = "00".repeat(32);
        let journal = format!(
            r#"{{"schema_version":1,"phase":"Prepared","target":{{"version":"1.2.3","executable_sha256":"{digest}"}},"previous":{{"version":"1.2.2","executable_sha256":"{digest}"}}}}"#
        );
        std::fs::write(root.path().join("ollama-update-state.json"), journal)
            .expect("recovery journal");
        assert!(abandoned_staging_dirs(root.path())
            .expect("inventory")
            .is_empty());
    }
}
