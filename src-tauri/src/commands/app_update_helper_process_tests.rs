use super::{run_helper_operation, SpawnedUpdateHelper};
use crate::app_exit::AppExitCoordinator;
use crate::services::process_identity::ProcessIdentity;
use crate::services::update_handoff::AppUpdateRuntime;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

/// Un enfant durable dont l'exécutable ne change pas après le lancement.
///
/// ProcessIdentity compare le chemin de l'exécutable pour écarter une
/// réutilisation de PID. Le lanceur `bin/python3.x` de Homebrew se ré-exécute
/// dans le binaire du framework : le chemin observé change quelques
/// millisecondes après le spawn et l'identité capturée devient périmée, alors
/// que le processus tourne toujours. Le vrai helper de mise à jour est un
/// binaire natif, sans ce détour.
fn long_lived_child() -> (PathBuf, &'static [&'static str]) {
    #[cfg(windows)]
    {
        let executable = crate::services::test_runtime::python().expect("runtime Python de test");
        (executable, &["-c", "import time; time.sleep(30)"])
    }
    #[cfg(not(windows))]
    {
        (PathBuf::from("/bin/sleep"), &["30"])
    }
}

fn test_helper() -> (SpawnedUpdateHelper, ProcessIdentity) {
    let (executable, arguments) = long_lived_child();
    let mut command = crate::services::background_command::new(&executable);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::services::process_tree::configure(&mut command);
    let child = command.spawn().expect("helper de test");
    let identity = ProcessIdentity::capture_child(child.id(), std::process::id(), &executable)
        .expect("identité du helper de test");
    (
        SpawnedUpdateHelper::from_test_child(child, identity.clone()),
        identity,
    )
}

fn is_running(identity: &ProcessIdentity) -> bool {
    let mut system = System::new();
    system.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(identity.pid())]),
        true,
    );
    identity.is_current(&system)
}

#[test]
fn update_helper_has_a_dedicated_process_configuration() {
    let executable = crate::services::test_runtime::python().expect("runtime Python de test");
    let mut command = crate::services::background_command::new(executable);
    command.args(["-c", "pass"]);
    crate::services::process_tree::configure_update_helper(&mut command);
    let status = command.status().expect("helper configuration");

    assert!(status.success());
}

#[tokio::test(flavor = "current_thread")]
async fn helper_io_does_not_block_the_async_worker() {
    let started = Instant::now();
    let blocker = std::thread::spawn(|| std::thread::sleep(Duration::from_millis(250)));
    let operation = run_helper_operation(move || {
        blocker.join().expect("blocking probe");
        7_u8
    });
    let sentinel = async {
        tokio::task::yield_now().await;
        started.elapsed()
    };

    let (result, sentinel_elapsed) = tokio::join!(operation, sentinel);

    assert_eq!(result.expect("blocking helper operation"), 7);
    assert!(sentinel_elapsed < Duration::from_millis(100));
}

#[test]
fn helper_not_transferred_is_stopped_and_reaped() {
    let (helper, identity) = test_helper();
    assert!(is_running(&identity));

    drop(helper);

    assert!(!is_running(&identity));
}

#[tokio::test]
async fn validated_transferred_helper_survives_update_shutdown() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let runtime = AppUpdateRuntime::new(coordinator.work_supervisor());
    let admission = runtime.try_admit().expect("update admission");
    let cancellation = admission.cancellation();
    let (helper, identity) = test_helper();

    helper
        .commit(runtime.handoff(), &cancellation)
        .expect("validated handoff");
    drop(admission);
    assert!(
        runtime
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(is_running(&identity));

    runtime.terminate_transferred_for_test();
    assert!(!is_running(&identity));
}

#[tokio::test]
async fn closing_rejects_and_reaps_an_untransferred_helper() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let runtime = AppUpdateRuntime::new(coordinator.work_supervisor());
    let admission = runtime.try_admit().expect("update admission");
    let cancellation = admission.cancellation();
    drop(admission);
    assert!(
        runtime
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    let (helper, identity) = test_helper();

    assert!(helper.commit(runtime.handoff(), &cancellation).is_err());
    assert!(!is_running(&identity));
}

#[tokio::test]
async fn handoff_preserves_exactly_one_validated_helper() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let runtime = AppUpdateRuntime::new(coordinator.work_supervisor());
    let admission = runtime.try_admit().expect("update admission");
    let cancellation = admission.cancellation();
    let (first, first_identity) = test_helper();
    let (second, second_identity) = test_helper();

    first
        .commit(runtime.handoff(), &cancellation)
        .expect("first validated helper");
    assert!(second.commit(runtime.handoff(), &cancellation).is_err());
    assert!(is_running(&first_identity));
    assert!(!is_running(&second_identity));

    runtime.terminate_transferred_for_test();
    assert!(!is_running(&first_identity));
}
