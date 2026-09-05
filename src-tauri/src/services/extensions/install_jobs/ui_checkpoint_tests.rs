use super::*;
use std::sync::Arc;

struct CheckUiJournal {
    fail: bool,
    journal: std::path::PathBuf,
}
impl InstallExecutor for CheckUiJournal {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        let fail = self.fail;
        let journal = self.journal.clone();
        Box::pin(async move {
            let token = uuid::Uuid::new_v4().simple().to_string();
            let source = tempfile::tempdir().unwrap();
            let entry = source.path().join("extension.mjs");
            std::fs::write(&entry, "export default {};").unwrap();
            let mut record = super::super::manifest::load_local(entry.to_str().unwrap())
                .unwrap()
                .record;
            record.manifest.id = format!("test.ui.{}", token);
            control
                .save(super::checkpoint::InstallCheckpoint {
                    version: super::checkpoint::FORMAT,
                    token: token.clone(),
                    record: Some(record.clone()),
                    ..Default::default()
                })
                .unwrap();
            let staging = super::super::ui_artifact_store::prepare_owned(&token).unwrap();
            let artifact = super::super::ui_artifact_tests::fixture(staging.output());
            let destination = super::super::ui_artifact_store::artifact_path(
                &record.manifest.id,
                &artifact.manifest_sha256,
            )
            .unwrap();
            if fail {
                std::fs::remove_file(&journal).unwrap();
                std::fs::create_dir(&journal).unwrap();
            }
            let result = super::super::ui_builder_build::commit_artifact(
                staging,
                &record,
                &artifact,
                Some(&control),
            );
            assert_eq!(result.is_err(), fail);
            assert_eq!(destination.exists(), !fail);
            if fail {
                std::fs::remove_dir(&journal).unwrap();
            } else {
                let saved = super::checkpoint::load(&journal).unwrap().unwrap();
                let owned = saved.jobs[0].checkpoint.as_ref().unwrap();
                assert_eq!(
                    owned
                        .record
                        .as_ref()
                        .unwrap()
                        .ui_artifact
                        .as_ref()
                        .unwrap()
                        .manifest_sha256,
                    artifact.manifest_sha256
                );
                super::cleanup::run(owned).unwrap();
                assert!(
                    !destination.exists(),
                    "recovery must find the unpublished artifact"
                );
                assert!(entry.exists(), "user source must survive cleanup");
            }
            super::cleanup::run(&control.saved().unwrap().unwrap()).unwrap();
            InstallOutcome {
                result: Err(InstallInterruption::Failed),
                cleanup_confirmed: true,
            }
        })
    }
}

#[tokio::test]
async fn ui_destination_is_durable_before_rename_and_write_failure_prevents_rename() {
    for fail in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let journal = root.path().join("jobs.json");
        let app = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let store = InstallJobStore::new(
            super::super::work_supervision::ExtensionWorkServices::new(app.work_supervisor()),
            Some(Arc::new(CheckUiJournal {
                fail,
                journal: journal.clone(),
            })),
            None,
        )
        .restore(journal);
        store
            .start(InstallRequest::Npm {
                locator: "example".into(),
            })
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while store.lock().unwrap().worker {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            store
                .work
                .stop_and_wait(std::time::Instant::now() + std::time::Duration::from_secs(2))
                .await
        );
    }
}
