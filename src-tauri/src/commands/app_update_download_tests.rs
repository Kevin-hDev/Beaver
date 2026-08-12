use super::write_verified_stream;
use crate::app_exit::AppExitCoordinator;
use crate::commands::app_update_install_temp::create_unique_temp_file;
use crate::services::update_handoff::AppUpdateRuntime;
use futures_util::{stream, StreamExt};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

fn expected_sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn runtime() -> AppUpdateRuntime {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    AppUpdateRuntime::new(coordinator.work_supervisor())
}

#[tokio::test]
async fn network_error_removes_its_partial_and_releases_admission() {
    let runtime = runtime();
    let (temporary, file) = create_unique_temp_file("beaver-update-network", ".part").unwrap();
    let path = temporary.path().to_path_buf();
    let result = runtime
        .run_download(move |cancel| async move {
            write_verified_stream(
                temporary,
                file,
                stream::iter([Ok(vec![1_u8]), Err(())]),
                2,
                &expected_sha256(&[1, 2]),
                &cancel,
                |_| {},
            )
            .await
        })
        .await;

    assert_eq!(result.unwrap_err(), "update-download-error");
    assert!(!path.exists());
    assert_eq!(runtime.diagnostics().active, 0);
}

#[tokio::test]
async fn cancellation_removes_its_partial_and_releases_admission() {
    let runtime = runtime();
    let (temporary, file) = create_unique_temp_file("beaver-update-cancel", ".part").unwrap();
    let path = temporary.path().to_path_buf();
    let task_runtime = runtime.clone();
    let (started, observed) = tokio::sync::oneshot::channel();
    let running = tokio::spawn(async move {
        let mut started = Some(started);
        task_runtime
            .run_download(move |cancel| async move {
                write_verified_stream(
                    temporary,
                    file,
                    stream::iter([Ok::<_, ()>(vec![1_u8])])
                        .chain(stream::pending::<Result<Vec<u8>, ()>>()),
                    2,
                    &expected_sha256(&[1, 2]),
                    &cancel,
                    move |_| {
                        if let Some(sender) = started.take() {
                            let _ = sender.send(());
                        }
                    },
                )
                .await
            })
            .await
    });

    observed.await.expect("one partial chunk written");
    assert!(
        runtime
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(running.await.unwrap().unwrap_err(), "update-download-error");
    assert!(!path.exists());
    assert_eq!(runtime.diagnostics().active, 0);
}

#[tokio::test]
async fn rejected_validation_removes_its_partial_and_releases_admission() {
    let runtime = runtime();
    let (temporary, file) = create_unique_temp_file("beaver-update-validation", ".part").unwrap();
    let path = temporary.path().to_path_buf();
    let result = runtime
        .run_download(move |cancel| async move {
            write_verified_stream(
                temporary,
                file,
                stream::iter([Ok::<_, ()>(vec![1_u8, 2])]),
                2,
                &expected_sha256(&[9, 9]),
                &cancel,
                |_| {},
            )
            .await
        })
        .await;

    assert_eq!(result.unwrap_err(), "update-download-error");
    assert!(!path.exists());
    assert_eq!(runtime.diagnostics().active, 0);
}
