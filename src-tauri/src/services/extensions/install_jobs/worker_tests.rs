use super::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

struct InstallFixture {
    root: tempfile::TempDir,
    store: InstallJobStore,
    id: String,
}
impl InstallFixture {
    fn new(script: &str) -> Self {
        crate::services::extensions::initialize_test_registry();
        let root = tempfile::tempdir().unwrap();
        let node = which::which("node").unwrap().canonicalize().unwrap();
        let cli = root.path().join("fake-npm.mjs");
        std::fs::write(&cli, script).unwrap();
        std::fs::write(root.path().join("user-source.mjs"), "export default {};").unwrap();
        let executor = super::executor::ProductionExecutor::for_test(
            super::super::npm_runner::NpmRunner::for_test(node.clone(), cli.clone()),
            super::super::ui_builder::UiBuildRuntime {
                node,
                builder: cli,
                directory: root.path().to_path_buf(),
            },
        );
        let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let work = super::super::work_supervision::ExtensionWorkServices::new(
            coordinator.work_supervisor(),
        );
        let store =
            InstallJobStore::new(work, Some(executor), None).restore(root.path().join("jobs.json"));
        Self {
            root,
            store,
            id: format!("test-{}", uuid::Uuid::new_v4().simple()),
        }
    }
    async fn wait(&self, id: &str, status: InstallStatus) -> InstallJobView {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let view = self
                    .store
                    .snapshot()
                    .unwrap()
                    .jobs
                    .into_iter()
                    .find(|job| job.id == id)
                    .unwrap();
                if view.status == status {
                    return view;
                }
                assert!(
                    !view.status.terminal(),
                    "unexpected terminal {:?}",
                    view.status
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }
    fn local(&self) -> PathBuf {
        let source = self.root.path().join("source");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(source.join("index.mjs"), "export default {};").unwrap();
        std::fs::write(source.join("beaver-extension.json"), serde_json::to_vec(&serde_json::json!({
            "id": self.id, "name": "Installer fixture", "version": "1.0.0", "beaverApi": "1",
            "runtime": "node", "main": "index.mjs", "access": "full", "apiLevel": "stable", "essential": false
        })).unwrap()).unwrap();
        source
    }
}

#[tokio::test]
async fn cancelling_real_npm_writer_confirms_process_death_before_cleaning_and_preserves_user_source(
) {
    // URL.pathname is not a filesystem path on Windows (or with escaped characters).
    let fixture = InstallFixture::new("import fs from 'node:fs';import path from 'node:path';import {fileURLToPath} from 'node:url';const root=path.dirname(fileURLToPath(import.meta.url));fs.writeFileSync(path.join(root,'pid'),String(process.pid));setInterval(()=>fs.appendFileSync('partial','x'),10);");
    let job = fixture
        .store
        .start(InstallRequest::Npm {
            locator: fixture.id.clone(),
        })
        .unwrap();
    let pid_file = fixture.root.path().join("pid");
    let pid: u32 = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(pid) = std::fs::read_to_string(&pid_file)
                .unwrap_or_default()
                .parse::<u32>()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        super::checkpoint::load(fixture.store.journal.as_ref().unwrap())
            .unwrap()
            .unwrap()
            .jobs[0]
            .checkpoint
            .as_ref()
            .unwrap()
            .native_process
            .is_some()
    );
    fixture.store.request_cancel(&job.id).unwrap();
    fixture.wait(&job.id, InstallStatus::Cancelled).await;
    assert!(!crate::services::owned_process::OwnedProcess::process_exists(pid));
    assert!(super::super::registry::find(&fixture.id).is_err());
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("user-source.mjs")).unwrap(),
        "export default {};"
    );
    {
        let state = fixture.store.lock().unwrap();
        let checkpoint = state.jobs[state.index(&job.id).unwrap()]
            .checkpoint
            .as_ref()
            .unwrap();
        assert!(!super::super::managed_store::root()
            .join(format!(".staging-{}", checkpoint.token))
            .exists());
    }
    assert!(
        fixture
            .store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

#[tokio::test]
async fn real_local_install_publishes_untrusted_record_and_preserves_source() {
    let fixture = InstallFixture::new("process.exit(9);");
    let source = fixture.local();
    let before = std::fs::read(source.join("index.mjs")).unwrap();
    let job = fixture
        .store
        .start(InstallRequest::Local {
            path: source.to_str().unwrap().into(),
        })
        .unwrap();
    let view = fixture.wait(&job.id, InstallStatus::Completed).await;
    assert_eq!(view.extension_id.as_deref(), Some(fixture.id.as_str()));
    let record = super::super::registry::find(&fixture.id).unwrap();
    assert!(!record.enabled && !record.trusted);
    assert_eq!(std::fs::read(source.join("index.mjs")).unwrap(), before);
    assert!(!source.join("node_modules").exists());
    super::super::registry::remove(&fixture.id).unwrap();
    assert!(
        fixture
            .store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

impl Drop for InstallFixture {
    fn drop(&mut self) {
        // A failed assertion must not abandon its real test producer.
        if let Ok(snapshot) = self.store.snapshot() {
            for job in snapshot.jobs.iter().filter(|job| !job.status.terminal()) {
                let _ = self.store.request_cancel(&job.id);
            }
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if self
                .store
                .snapshot()
                .is_ok_and(|snapshot| snapshot.jobs.iter().all(|job| job.status.terminal()))
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
