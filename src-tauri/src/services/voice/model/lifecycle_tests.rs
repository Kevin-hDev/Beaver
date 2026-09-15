use super::{
    lifecycle::ModelLifecycle,
    lifecycle_maintenance::seconds,
    lifecycle_state::{lock, ModelKey},
};
use crate::app_exit::AppExitCoordinator;
use crate::services::voice::download::{InstallationReceipt, ReceiptFile};
use crate::services::voice::types::VoiceUnloadDelay;
use sha2::{Digest, Sha256};

#[test]
fn every_unload_setting_has_one_exact_deadline() {
    assert_eq!(seconds(VoiceUnloadDelay::Immediately), Some(0));
    assert_eq!(seconds(VoiceUnloadDelay::OneMinute), Some(60));
    assert_eq!(seconds(VoiceUnloadDelay::TwoMinutes), Some(120));
    assert_eq!(seconds(VoiceUnloadDelay::FiveMinutes), Some(300));
    assert_eq!(seconds(VoiceUnloadDelay::FifteenMinutes), Some(900));
    assert_eq!(seconds(VoiceUnloadDelay::OnExit), None);
}

#[tokio::test]
async fn model_work_tracks_maintenance_and_one_asr_load() {
    let exit = AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(exit.work_supervisor());
    assert_eq!(lifecycle.work.diagnostics().active, 1);

    let loading = lifecycle.work.try_admit().unwrap();
    let cancellation = loading.cancellation();
    assert_eq!(lifecycle.work.diagnostics().active, 2);
    assert!(lifecycle.work.try_admit().is_err());

    lifecycle.begin_closing();
    assert!(cancellation.is_cancelled());
    drop(loading);
    assert!(
        lifecycle
            .stop_and_wait(std::time::Instant::now() + std::time::Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn occupied_asr_and_shared_vad_cannot_be_removed() {
    let exit = AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(exit.work_supervisor());
    let data = tempfile::tempdir().unwrap();
    let key = ModelKey::fixture("parakeet", "silero-vad");
    lock(&lifecycle.inner.state).occupied = Some(key);

    assert_eq!(
        lifecycle.remove(data.path(), "parakeet").unwrap_err(),
        "model-download-model-busy"
    );
    assert_eq!(
        lifecycle.remove(data.path(), "silero-vad").unwrap_err(),
        "model-download-model-busy"
    );
    lock(&lifecycle.inner.state).occupied = None;
    lifecycle.remove(data.path(), "parakeet").unwrap();

    lifecycle.begin_closing();
    assert!(
        lifecycle
            .stop_and_wait(std::time::Instant::now() + std::time::Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn reservation_blocks_model_change_and_reinstall_invalidates_verification() {
    let exit = AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(exit.work_supervisor());
    let first = ModelKey::fixture("parakeet", "silero-vad");
    let second = ModelKey::fixture("cohere", "silero-vad");

    lock(&lifecycle.inner.state)
        .verified
        .insert(("parakeet".into(), "rev-1".into()));
    lifecycle.begin_acquire(&first).unwrap();
    assert!(lifecycle.begin_acquire(&second).is_err());
    assert!(lifecycle.invalidate_installation("parakeet").is_err());
    assert!(lock(&lifecycle.inner.state)
        .verified
        .contains(&("parakeet".into(), "rev-1".into())));

    lock(&lifecycle.inner.state).occupied = None;
    lifecycle.invalidate_installation("parakeet").unwrap();
    assert!(!lock(&lifecycle.inner.state)
        .verified
        .contains(&("parakeet".into(), "rev-1".into())));

    lifecycle.begin_closing();
    assert!(
        lifecycle
            .stop_and_wait(std::time::Instant::now() + std::time::Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn verification_is_cached_until_reinstall_or_a_new_process() {
    let data = tempfile::tempdir().unwrap();
    let bytes = b"verified model";
    let digest = hex::encode(Sha256::digest(bytes));
    let receipt = InstallationReceipt {
        version: 1,
        entry_id: "parakeet".into(),
        revision: "a".repeat(40),
        installed_bytes: bytes.len() as u64,
        files: vec![ReceiptFile {
            path: "model.onnx".into(),
            bytes: bytes.len() as u64,
            sha256: digest,
        }],
    };
    let model = receipt.install_dir(data.path()).join("model.onnx");
    std::fs::create_dir_all(model.parent().unwrap()).unwrap();
    std::fs::write(&model, bytes).unwrap();

    let first_exit = AppExitCoordinator::initialize().unwrap();
    let first = ModelLifecycle::new(first_exit.work_supervisor());
    first
        .verify_once(data.path(), [&receipt, &receipt])
        .unwrap();
    std::fs::write(&model, b"corrupt model!").unwrap();
    first
        .verify_once(data.path(), [&receipt, &receipt])
        .unwrap();
    first.invalidate_installation("parakeet").unwrap();
    assert!(first
        .verify_once(data.path(), [&receipt, &receipt])
        .is_err());

    let second_exit = AppExitCoordinator::initialize().unwrap();
    let second = ModelLifecycle::new(second_exit.work_supervisor());
    assert!(second
        .verify_once(data.path(), [&receipt, &receipt])
        .is_err());

    first.begin_closing();
    second.begin_closing();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    assert!(first.stop_and_wait(deadline).await);
    assert!(second.stop_and_wait(deadline).await);
}
