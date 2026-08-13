#[cfg(test)]
mod tests {
    use crate::app_exit::AppExitCoordinator;
    use crate::services::model_downloads_store::{ModelDownloadManager, ProgressUpdate};
    use crate::services::model_downloads_types::{
        ModelDownloadKind, ModelDownloadPhase, ModelDownloadStatus, MAX_PENDING_DOWNLOADS,
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::{Duration, Instant};

    fn test_manager() -> ModelDownloadManager {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        ModelDownloadManager::new(coordinator.work_supervisor())
    }

    #[tokio::test]
    async fn queues_a_second_download_while_one_is_running() {
        let manager = test_manager();
        let (first, first_runner) = manager
            .start(ModelDownloadKind::Ollama, "llama3:latest".into(), false)
            .await
            .unwrap();
        assert!(first_runner.is_some());
        assert_eq!(first.status, ModelDownloadStatus::Running);

        let (second, second_runner) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();
        assert!(second_runner.is_none());
        assert_eq!(second.status, ModelDownloadStatus::Queued);
        assert_eq!(manager.list().await.len(), 2);
    }

    #[tokio::test]
    async fn cancel_marks_token_then_finish_sets_cancelled_status() {
        let manager = test_manager();
        let (state, cancel) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();

        manager.cancel(&state.id).await.unwrap();
        assert!(cancel.unwrap().0.is_cancelled());

        let states = manager
            .finish(&state.id, ModelDownloadStatus::Cancelled, None)
            .await;
        assert_eq!(states[0].status, ModelDownloadStatus::Cancelled);
    }

    #[tokio::test]
    async fn completed_download_is_removed_before_next_start() {
        let manager = test_manager();
        let (state, _) = manager
            .start(ModelDownloadKind::Ollama, "llama3:latest".into(), false)
            .await
            .unwrap();
        manager
            .progress(
                &state.id,
                ProgressUpdate {
                    phase: ModelDownloadPhase::Downloading,
                    downloaded: 1,
                    total: 2,
                    percent: 50,
                },
            )
            .await;
        manager
            .finish(&state.id, ModelDownloadStatus::Completed, None)
            .await;
        assert!(manager.complete_and_activate_next().await.is_none());

        let (next, runner) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();
        assert!(runner.is_some());
        let states = manager.list().await;
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].id, next.id);
    }

    #[tokio::test]
    async fn finishing_the_active_download_starts_the_next_queued_item() {
        let manager = test_manager();
        let (first, _) = manager
            .start(ModelDownloadKind::Ollama, "llama3:latest".into(), false)
            .await
            .unwrap();
        let (second, _) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();

        manager
            .finish(&first.id, ModelDownloadStatus::Completed, None)
            .await;
        let (activated, _) = manager.complete_and_activate_next().await.unwrap();

        assert_eq!(activated.id, second.id);
        assert_eq!(activated.status, ModelDownloadStatus::Running);
    }

    #[tokio::test]
    async fn queue_is_bounded() {
        let manager = test_manager();
        for index in 0..MAX_PENDING_DOWNLOADS {
            manager
                .start(ModelDownloadKind::Ollama, format!("model-{index}"), false)
                .await
                .unwrap();
        }

        let error = manager
            .start(ModelDownloadKind::Ollama, "one-too-many".into(), false)
            .await
            .unwrap_err();

        assert_eq!(error, "model-download-queue-full");
        assert_eq!(manager.list().await.len(), MAX_PENDING_DOWNLOADS);
    }

    #[tokio::test]
    async fn a_queued_download_can_be_cancelled_before_it_starts() {
        let manager = test_manager();
        let (first, _) = manager
            .start(ModelDownloadKind::Ollama, "first".into(), false)
            .await
            .unwrap();
        let (queued, _) = manager
            .start(ModelDownloadKind::Forecast, "second".into(), false)
            .await
            .unwrap();

        let states = manager.cancel(&queued.id).await.unwrap();
        assert_eq!(
            states
                .iter()
                .find(|state| state.id == queued.id)
                .unwrap()
                .status,
            ModelDownloadStatus::Cancelled,
        );
        manager
            .finish(&first.id, ModelDownloadStatus::Completed, None)
            .await;
        assert!(manager.complete_and_activate_next().await.is_none());
    }

    #[tokio::test]
    async fn app_exit_cancels_running_and_queued_downloads() {
        let manager = test_manager();
        let (_, running_cancel) = manager
            .start(ModelDownloadKind::Ollama, "first".into(), false)
            .await
            .unwrap();
        let (_, queued_cancel) = manager
            .start(ModelDownloadKind::Forecast, "second".into(), false)
            .await
            .unwrap();

        manager.cancel_all().await;

        assert!(running_cancel.unwrap().0.is_cancelled());
        assert!(queued_cancel.is_none());
        assert_eq!(
            manager.list().await[1].status,
            ModelDownloadStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn shutdown_waits_for_the_worker_and_permanently_refuses_new_downloads() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = ModelDownloadManager::new(coordinator.work_supervisor());
        let (_, runner) = manager
            .start(ModelDownloadKind::Ollama, "first".into(), false)
            .await
            .unwrap();
        let (_, admission) = runner.expect("download worker");
        let finished = Arc::new(AtomicBool::new(false));
        let task_finished = Arc::clone(&finished);
        admission
            .spawn(move |cancel| async move {
                cancel.cancelled().await;
                tokio::time::sleep(Duration::from_millis(20)).await;
                task_finished.store(true, Ordering::Release);
            })
            .unwrap();

        assert!(
            manager
                .stop_and_wait(Instant::now() + Duration::from_secs(1))
                .await
        );
        assert!(finished.load(Ordering::Acquire));
        assert_eq!(
            manager
                .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
                .await
                .unwrap_err(),
            "service-shutting-down"
        );
    }

    #[tokio::test]
    async fn production_completion_releases_an_empty_worker_for_the_next_download() {
        let manager = test_manager();
        let (first, runner) = manager
            .start(ModelDownloadKind::Ollama, "first".into(), false)
            .await
            .unwrap();
        manager
            .finish(&first.id, ModelDownloadStatus::Completed, None)
            .await;
        assert!(runner.is_some());
        assert!(manager.complete_and_activate_next().await.is_none());
        drop(runner);

        let (second, second_runner) = manager
            .start(ModelDownloadKind::Forecast, "chronos-tiny".into(), false)
            .await
            .unwrap();

        assert!(second_runner.is_some());
        assert_eq!(manager.list().await[0].id, second.id);
    }
}
