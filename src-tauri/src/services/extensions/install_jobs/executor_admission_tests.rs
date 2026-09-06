use super::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[tokio::test]
async fn saturated_producer_admission_never_claims_retained_artifacts_are_clean() {
    for has_checkpoint in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let node = which::which("node").unwrap();
        let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let work = crate::services::extensions::work_supervision::ExtensionWorkServices::new(
            coordinator.work_supervisor(),
        );
        let executor = ProductionExecutor {
            npm: NpmRunner::for_test(node.clone(), root.path().join("never-run.mjs")),
            ui: UiBuildRuntime {
                node,
                builder: root.path().join("never-run.mjs"),
                directory: root.path().to_owned(),
            },
        };
        let store = crate::services::extensions::install_jobs::InstallJobStore::new(
            work.clone(),
            Some(Arc::new(executor)),
            None,
        );
        store
            .start(InstallRequest::Npm {
                locator: "saturated-fixture".into(),
            })
            .unwrap();
        if has_checkpoint {
            store.lock().unwrap().jobs[0].checkpoint = Some(Default::default());
        }
        // Either the app-wide or the service bound can be reached first.
        let mut admissions = Vec::new();
        for _ in 0..crate::services::extensions::work_supervision::MAX_EXTENSION_OPERATIONS {
            match work.try_admit_operation() {
                Ok(admission) => admissions.push(admission),
                Err(crate::services::extensions::work_supervision::ExtensionWorkAdmissionError::Busy) => break,
                Err(error) => panic!("unexpected admission refusal: {error:?}"),
            }
        }
        assert!(matches!(
            work.try_admit_operation(),
            Err(crate::services::extensions::work_supervision::ExtensionWorkAdmissionError::Busy)
        ));
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if store.snapshot().unwrap().jobs[0].status.terminal() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        {
            let state = store.lock().unwrap();
            assert_eq!(
                state.jobs[0].view.status,
                crate::services::extensions::install_jobs::InstallStatus::Failed
            );
            assert_eq!(state.jobs[0].clean, !has_checkpoint);
            assert_eq!(state.jobs[0].checkpoint.is_some(), has_checkpoint);
        }
        drop(admissions);
        assert!(
            work.stop_and_wait(Instant::now() + Duration::from_secs(2))
                .await
        );
    }
}
