use super::durable_fs::platform_fs;
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::journal_store::OllamaJournalStore;
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use crate::services::paths::ollama_paths;
use std::sync::Arc;

#[tokio::test]
async fn cancelled_update_removes_both_partial_staging_directories_in_one_cleanup() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    std::fs::create_dir_all(&paths.update_staging).expect("update staging");
    std::fs::write(paths.update_staging.join("partial"), b"partial").expect("update bytes");
    std::fs::create_dir_all(&paths.archive_staging).expect("archive staging");
    std::fs::write(paths.archive_staging.join("archive.part"), b"partial").expect("archive bytes");
    std::fs::write(&paths.process_receipt, b"owned-sidecar").expect("process receipt sentinel");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");

    super::cancel_cleanup::cleanup_with(Arc::new(platform_fs()), paths.clone(), models)
        .await
        .expect("cancel cleanup");

    for path in [
        paths.update_staging,
        paths.archive_staging,
        paths.uncommitted_staging_delete,
        paths.archive_failed,
    ] {
        assert!(!path.exists(), "{} must be removed", path.display());
    }
    assert_eq!(
        std::fs::read(&paths.process_receipt).expect("process receipt preserved"),
        b"owned-sidecar"
    );
}

#[tokio::test]
async fn cancelled_update_never_deletes_staging_after_a_journal_exists() {
    let root = tempfile::tempdir_in(".").expect("layout root");
    let paths = ollama_paths(root.path());
    std::fs::create_dir_all(&paths.update_staging).expect("update staging");
    let models_path = root.path().join("models");
    std::fs::create_dir_all(&models_path).expect("models directory");
    let models = NativePathIdentityResolver
        .canonical_directory(&models_path)
        .expect("models identity");
    let fs = Arc::new(platform_fs());
    let store = OllamaJournalStore::new(Arc::clone(&fs), paths.clone());
    let fingerprint = super::fingerprint::BundleFingerprint {
        version: super::fingerprint::OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: super::fingerprint::Sha256Digest::from_hex(&"11".repeat(32))
            .expect("digest"),
    };
    store
        .write_new(&OllamaTransactionJournal::new(
            OllamaJournalState::Prepared {
                target: fingerprint.clone(),
                previous: fingerprint,
            },
        ))
        .await
        .expect("journal");

    let result = super::cancel_cleanup::cleanup_with(fs, paths.clone(), models).await;

    assert_eq!(
        result,
        Err(super::error::OllamaErrorCode::OllamaUpdateRecoveryRequired)
    );
    assert!(paths.update_staging.exists());
}
