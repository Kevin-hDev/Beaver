use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

struct Publishing {
    cancel_first: bool,
    published: Arc<AtomicBool>,
    lose_journal: bool,
}
impl InstallExecutor for Publishing {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        let cancel_first = self.cancel_first;
        let published = self.published.clone();
        let lose_journal = self.lose_journal;
        Box::pin(async move {
            if cancel_first {
                control.store.request_cancel(&control.id).unwrap();
            }
            let result = control.publish(|| {
                published.store(true, Ordering::SeqCst);
                Ok("test.publication".into())
            });
            if result.is_ok() {
                assert_eq!(
                    control.store.request_cancel(&control.id).unwrap().status,
                    InstallStatus::Completed
                );
            }
            if lose_journal {
                control.store.lock().unwrap().durable_error = true;
                return super::executor::complete(&control, Default::default(), result);
            }
            InstallOutcome {
                result,
                cleanup_confirmed: true,
            }
        })
    }
}

#[tokio::test]
async fn cancellation_and_publication_have_exactly_one_winner() {
    for cancel_first in [true, false] {
        let app = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let published = Arc::new(AtomicBool::new(false));
        let store = InstallJobStore::new(
            super::super::work_supervision::ExtensionWorkServices::new(app.work_supervisor()),
            Some(Arc::new(Publishing {
                cancel_first,
                published: published.clone(),
                lose_journal: false,
            })),
            None,
        );
        let job = store
            .start(InstallRequest::Npm {
                locator: "example".into(),
            })
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !store.snapshot().unwrap().jobs[0].status.terminal() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(published.load(Ordering::SeqCst), !cancel_first);
        assert_eq!(
            store.snapshot().unwrap().jobs[0].status,
            if cancel_first {
                InstallStatus::Cancelled
            } else {
                InstallStatus::Completed
            }
        );
        assert_eq!(store.snapshot().unwrap().jobs[0].id, job.id);
        assert!(
            store
                .work
                .stop_and_wait(Instant::now() + Duration::from_secs(2))
                .await
        );
    }
}

#[tokio::test]
async fn journal_failure_after_publication_preserves_completed_and_blocks_new_work() {
    let app = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let published = Arc::new(AtomicBool::new(false));
    let store = InstallJobStore::new(
        super::super::work_supervision::ExtensionWorkServices::new(app.work_supervisor()),
        Some(Arc::new(Publishing {
            cancel_first: false,
            published: published.clone(),
            lose_journal: true,
        })),
        None,
    );
    let job = store
        .start(InstallRequest::Npm {
            locator: "example".into(),
        })
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while store.lock().unwrap().worker {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(published.load(Ordering::SeqCst));
    assert_eq!(
        store.snapshot().unwrap().jobs[0].status,
        InstallStatus::Completed
    );
    assert!(!store.lock().unwrap().jobs[0].clean);
    assert!(store
        .start(InstallRequest::Npm {
            locator: "another".into()
        })
        .is_err());
    assert_eq!(
        store.request_cancel(&job.id).unwrap().status,
        InstallStatus::Completed
    );
    assert!(
        store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}
