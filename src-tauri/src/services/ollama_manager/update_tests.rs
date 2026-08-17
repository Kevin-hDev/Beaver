#[path = "update_order_tests.rs"]
mod order_tests;
#[path = "update_recovery_tests.rs"]
mod recovery_tests;
#[path = "update_test_support.rs"]
mod support;

use super::error::OllamaErrorCode;
use super::manager::OllamaManager;
use super::update::UpdateRequest;
use crate::app_exit::AppExitCoordinator;

#[tokio::test]
async fn manager_exposes_update_entrypoint() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let root = tempfile::tempdir().expect("temporary root");
    let result = manager
        .update(UpdateRequest::for_test(root.path().to_path_buf()))
        .await;
    assert_ne!(result, Err(OllamaErrorCode::OllamaInternal));
}
