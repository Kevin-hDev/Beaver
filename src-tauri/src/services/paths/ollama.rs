#![allow(dead_code)]

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaPaths {
    pub active: PathBuf,
    pub legacy_staging: PathBuf,
    pub legacy_backup: PathBuf,
    pub failed: PathBuf,
    pub install_staging: PathBuf,
    pub archive_staging: PathBuf,
    pub archive_failed: PathBuf,
    pub update_staging: PathBuf,
    pub backup: PathBuf,
    pub backup_delete: PathBuf,
    pub failed_delete: PathBuf,
    pub journal: PathBuf,
    pub journal_tmp: PathBuf,
    pub migration_marker: PathBuf,
    pub migration_marker_tmp: PathBuf,
    pub process_receipt: PathBuf,
    pub probe_models: PathBuf,
}

pub fn bundle_receipt_path(bundle_root: &Path) -> PathBuf {
    bundle_root.join("ollama-bundle-receipt.json")
}

pub fn bundle_receipt_tmp_path(bundle_root: &Path) -> PathBuf {
    bundle_root.join("ollama-bundle-receipt.tmp")
}

pub fn ollama_paths(data_dir: &Path) -> OllamaPaths {
    OllamaPaths {
        active: data_dir.join("ollama-bundle"),
        legacy_staging: data_dir.join("ollama-bundle-staging"),
        legacy_backup: data_dir.join("ollama-bundle-old"),
        failed: data_dir.join("ollama-bundle-failed"),
        install_staging: data_dir.join("ollama-bundle-install-staging"),
        archive_staging: data_dir.join("ollama-bundle-install-staging-archives"),
        archive_failed: data_dir.join("ollama-bundle-install-staging-archives-failed"),
        update_staging: data_dir.join("ollama-bundle-update-staging"),
        backup: data_dir.join("ollama-bundle-backup"),
        backup_delete: data_dir.join("ollama-bundle-backup-delete"),
        failed_delete: data_dir.join("ollama-bundle-failed-delete"),
        journal: data_dir.join("ollama-update-state.json"),
        journal_tmp: data_dir.join("ollama-update-state.tmp"),
        migration_marker: data_dir.join("ollama-layout-migration.json"),
        migration_marker_tmp: data_dir.join("ollama-layout-migration.tmp"),
        process_receipt: data_dir.join("ollama-process-receipt.json"),
        probe_models: data_dir.join("ollama-probe-models"),
    }
}
