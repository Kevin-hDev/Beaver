use crate::app_exit::AppExitCoordinator;
use crate::services::ollama_manager::{
    CancelOutcome, OllamaManager, OllamaProgressStage, OllamaVersion, OperationState,
};
use tokio_util::sync::CancellationToken;

#[test]
fn valid_semver_accepted() {
    assert!(OllamaVersion::parse("0.23.1").is_ok());
    assert!(OllamaVersion::parse("1.0.0").is_ok());
    assert!(OllamaVersion::parse("0.30.0-rc3").is_ok());
    assert!(OllamaVersion::parse("2.1.0-beta.1").is_ok());
}

#[test]
fn invalid_semver_rejected() {
    assert!(OllamaVersion::parse("").is_err());
    assert!(OllamaVersion::parse("1.0").is_err());
    assert!(OllamaVersion::parse("abc").is_err());
    assert!(OllamaVersion::parse("1.0.0/../../evil").is_err());
    assert!(OllamaVersion::parse("1.0.0%0d%0aHeader: inject").is_err());
    assert!(OllamaVersion::parse("1.0.0\nmalicious").is_err());
    assert!(OllamaVersion::parse("v1.0.0").is_err());
}

#[test]
fn fallback_install_version_is_current_supported_release() {
    let fallback = crate::services::ollama_manager::release_source::fallback_version()
        .unwrap()
        .to_string();
    assert_eq!(fallback, "0.32.1");
    assert!(OllamaVersion::parse(&fallback).is_ok());
}

#[test]
fn every_typed_progress_stage_has_one_stable_channel_status() {
    let cases = [
        (OllamaProgressStage::Preparing, "preparing"),
        (OllamaProgressStage::Downloading, "downloading"),
        (OllamaProgressStage::Verifying, "verifying"),
        (OllamaProgressStage::Extracting, "extracting"),
        (OllamaProgressStage::Validating, "validating"),
        (OllamaProgressStage::Committing, "committing"),
        (OllamaProgressStage::Starting, "starting"),
        (OllamaProgressStage::Recovering, "recovering"),
        (OllamaProgressStage::RollingBack, "rolling_back"),
        (OllamaProgressStage::Cleaning, "cleaning"),
    ];
    for (stage, expected) in cases {
        assert_eq!(super::ollama_setup::progress_status(stage), expected);
    }
}

#[tokio::test]
async fn cancel_active_setup_cancels_manager_token() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let token = CancellationToken::new();
    let operation = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("operation admission");
    manager.set_operation_cancellation(token.clone());

    assert_eq!(manager.cancel_operation().await, CancelOutcome::Cancelled);

    assert!(token.is_cancelled());
    manager.clear_operation_cancellation();
    drop(operation);
}
