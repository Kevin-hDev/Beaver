use super::InstallerService;
use crate::install_trace::TraceSession;
use crate::launch_args::{LaunchContext, PinnedRelease};
use crate::runtime::{InstallerRuntime, PlatformKind};
use crate::temp_ownership::{OwnedTempRun, OWNER_MARKER};
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[cfg(unix)]
#[test]
fn cleanup_failure_is_attempted_only_once() {
    use std::os::unix::fs::symlink;

    let run_id = "0123456789abcdef0123456789abcdef";
    let root = std::env::temp_dir().join(format!("beaver-exit-test-{}", std::process::id()));
    let work_dir = root.join(format!("beaver-install-{run_id}"));
    fs::create_dir_all(&work_dir).unwrap();
    fs::write(
        work_dir.join(OWNER_MARKER),
        format!(r#"{{"schema":1,"runId":"{run_id}"}}"#),
    )
    .unwrap();
    symlink(&root, work_dir.join("unsafe-link")).unwrap();
    let run = OwnedTempRun::adopt(&root, &work_dir, run_id).unwrap();
    let service = InstallerService {
        launch: LaunchContext {
            run_id: run_id.into(),
            work_dir: work_dir.clone(),
            release: PinnedRelease {
                version: "1.2.3".into(),
                app_asset_name: "Beaver_1.2.3_aarch64.dmg".into(),
                app_asset_size: 1,
                app_asset_sha256: "0".repeat(64),
            },
        },
        run,
        destination: Mutex::new(root.clone()),
        runtime: Arc::new(InstallerRuntime::new(
            "1.2.3",
            root.to_str().unwrap(),
            None,
            false,
            PlatformKind::Macos,
        )),
        trace: TraceSession::new(),
        cleanup_done: AtomicBool::new(false),
    };

    assert!(service.shutdown().is_err());
    assert!(service.shutdown().is_ok());

    fs::remove_file(work_dir.join("unsafe-link")).unwrap();
    fs::remove_dir_all(root).unwrap();
}
