use super::model_downloads_store::ModelDownloadManager;
use super::model_downloads_types::{ModelDownloadKind, ModelDownloadStatus};
use crate::app_exit::AppExitCoordinator;
use crate::services::voice::download::{checkpoint_path, save_checkpoint, Checkpoint};
use std::time::{Duration, Instant};

fn manager() -> ModelDownloadManager {
    let exit = AppExitCoordinator::initialize().expect("exit coordinator");
    ModelDownloadManager::new(exit.work_supervisor())
}

fn checkpoint(entry: &crate::services::voice::download::VoiceCatalogEntry) -> Checkpoint {
    let mut checkpoint = Checkpoint::new(
        entry.id.clone(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .expect("checkpoint");
    checkpoint.durable_bytes = 50;
    checkpoint
}

#[tokio::test]
async fn restored_voice_checkpoint_stays_suspended_until_manual_resume() {
    let data = tempfile::tempdir().expect("data");
    let resource_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let entry = crate::services::voice::download::load_catalog(resource_dir)
        .unwrap()
        .entries
        .remove(0);
    save_checkpoint(
        &checkpoint_path(data.path(), &entry.id),
        &checkpoint(&entry),
    )
    .expect("save checkpoint");
    let manager = manager();
    manager.restore_voice_checkpoints(data.path(), resource_dir);

    let suspended = manager.list().await.remove(0);
    assert_eq!(suspended.status, ModelDownloadStatus::Suspended);
    assert_eq!(suspended.downloaded, 50);

    let (other, _) = manager
        .start(ModelDownloadKind::Ollama, "tiny".into(), false)
        .await
        .expect("start unrelated transfer");
    manager
        .finish(&other.id, ModelDownloadStatus::Completed, None, None)
        .await;
    assert!(manager.complete_and_activate_next().await.is_none());
    assert_eq!(
        manager.state(&suspended.id).await.unwrap().status,
        ModelDownloadStatus::Suspended
    );

    let (resumed, runner) = manager.resume(&suspended.id).await.expect("manual resume");
    assert_eq!(resumed.status, ModelDownloadStatus::Running);
    assert!(runner.is_some());
}

#[tokio::test]
async fn shutdown_suspends_voice_work_instead_of_cancelling_it() {
    let manager = manager();
    let (state, runner) = manager
        .start(
            ModelDownloadKind::Voice,
            "parakeet-tdt-v3-int8".into(),
            false,
        )
        .await
        .unwrap();
    let (cancel, admission) = runner.unwrap();
    admission
        .spawn(move |shutdown| async move {
            shutdown.cancelled().await;
        })
        .unwrap();

    assert!(
        manager
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(cancel.is_cancelled());
    assert_eq!(
        manager.state(&state.id).await.unwrap().status,
        ModelDownloadStatus::Suspended
    );
}

#[tokio::test]
#[ignore = "downloads and extracts the real smallest ASR catalogue entry"]
async fn real_smallest_asr_download_resumes_the_same_revision() {
    let resource_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let entry = crate::services::voice::download::load_catalog(resource_dir)
        .unwrap()
        .entries
        .into_iter()
        .filter(|entry| entry.role == crate::services::voice::download::VoiceModelRole::Asr)
        .min_by_key(|entry| entry.archive.bytes)
        .unwrap();
    let data = tempfile::tempdir().unwrap();
    crate::services::voice::download::ensure_disk_available(data.path(), &entry).unwrap();
    let cancel = tokio_util::sync::CancellationToken::new();
    let progress_cancel = cancel.clone();
    let interrupted = crate::services::voice::download::download_archive(
        &entry,
        &entry.id,
        data.path(),
        &cancel,
        move |downloaded, _| {
            if downloaded >= 4 * 1024 * 1024 {
                progress_cancel.cancel();
            }
        },
    )
    .await;
    assert_eq!(interrupted.unwrap_err(), "cancelled");
    let before = crate::services::voice::download::discover_checkpoints(data.path())
        .remove(0)
        .durable_bytes;

    let archive = crate::services::voice::download::download_archive(
        &entry,
        &entry.id,
        data.path(),
        &tokio_util::sync::CancellationToken::new(),
        |_, _| {},
    )
    .await
    .unwrap();
    let after = crate::services::voice::download::discover_checkpoints(data.path())
        .remove(0)
        .durable_bytes;
    let receipt = tokio::task::spawn_blocking({
        let entry = entry.clone();
        let data = data.path().to_path_buf();
        move || crate::services::voice::download::install_archive(&entry, &archive, &data)
    })
    .await
    .unwrap()
    .unwrap();
    println!(
        "entry={} revision={} durable_before={} durable_after={} files={}",
        entry.id,
        entry.revision,
        before,
        after,
        receipt.files.len()
    );
    assert!(before >= 4 * 1024 * 1024);
    assert_eq!(after, entry.archive.bytes);
}
