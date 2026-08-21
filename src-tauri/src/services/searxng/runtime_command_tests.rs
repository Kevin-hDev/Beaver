use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::Duration;
use tokio::process::Command;

use super::runtime_command::{run_runtime_command, RuntimeStage};

static LOG_GUARD: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(windows)]
const INHERITED_PIPE_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(not(windows))]
const INHERITED_PIPE_TIMEOUT: Duration = Duration::from_millis(50);

#[tokio::test]
async fn successful_runtime_command_drains_both_streams() {
    let _guard = LOG_GUARD.lock().await;
    let result = run_fixture(
        "import sys; print('stdout-ready'); print('stderr-ready', file=sys.stderr)",
        Duration::from_secs(1),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn non_zero_command_keeps_both_bounded_output_tails() {
    let _guard = LOG_GUARD.lock().await;
    let result = run_fixture(
        "import sys; print('o' * 2000000); print('stdout-tail'); print('e' * 2000000, file=sys.stderr); print('stderr-tail', file=sys.stderr); raise SystemExit(7)",
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let log = std::fs::read_to_string(super::paths::runtime_log_path()).expect("runtime log");
    assert!(log.contains("stdout-tail"));
    assert!(log.contains("stderr-tail"));
    assert!(
        std::fs::metadata(super::paths::runtime_log_path())
            .expect("runtime log metadata")
            .len()
            <= 16_384
    );
}

#[tokio::test]
async fn timeout_terminates_the_owned_process_and_bounds_the_log() {
    let _guard = LOG_GUARD.lock().await;
    let started = std::time::Instant::now();
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let code = format!(
        "import os,sys,time; open({:?}, 'w').write(str(os.getpid())); print('x' * 50000); time.sleep(30)",
        pid_file.path()
    );
    let result = run_fixture(&code, Duration::from_millis(50)).await;

    assert_eq!(result.unwrap_err().category(), "timeout");
    assert!(started.elapsed() < Duration::from_millis(900));
    let pid = std::fs::read_to_string(pid_file.path())
        .expect("child pid")
        .parse::<u32>()
        .expect("numeric child pid");
    assert!(wait_until_process_is_gone(pid).await);
    assert!(
        std::fs::metadata(super::paths::runtime_log_path())
            .expect("runtime log metadata")
            .len()
            <= 16_384
    );
}

#[tokio::test]
async fn timeout_terminates_the_explicit_process_tree_not_only_the_root() {
    let _guard = LOG_GUARD.lock().await;
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let code = format!(
        "import os,subprocess,sys,time\nchild=subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)\nopen({:?}, 'w').write(f'{{os.getpid()}} {{child.pid}}')\ntime.sleep(30)",
        pid_file.path()
    );

    let result = run_fixture(&code, Duration::from_millis(80)).await;

    assert_eq!(result.unwrap_err().category(), "timeout");
    let pids = std::fs::read_to_string(pid_file.path()).expect("tree pids");
    let pids: Vec<u32> = pids
        .split_whitespace()
        .map(|pid| pid.parse().expect("numeric pid"))
        .collect();
    assert_eq!(pids.len(), 2);
    for pid in pids {
        assert!(wait_until_process_is_gone(pid).await, "pid {pid} survived");
    }
}

#[tokio::test]
async fn expired_deadline_does_not_attempt_to_spawn_a_process() {
    let _guard = LOG_GUARD.lock().await;
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let mut command = Command::new("/definitely-not-a-runtime-command");

    let result = run_runtime_command(
        &mut command,
        RuntimeStage::ValidateImports,
        tokio::time::Instant::now() - Duration::from_millis(1),
        &admission.cancellation(),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "timeout");
}

#[tokio::test]
async fn inherited_pipes_from_a_descendant_obey_the_global_deadline() {
    let _guard = LOG_GUARD.lock().await;
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let parent = format!(
        "import os,subprocess,sys\nstdout=os.dup(1)\nstderr=os.dup(2)\nos.set_inheritable(stdout, True)\nos.set_inheritable(stderr, True)\nchild=subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], stdout=stdout, stderr=stderr, close_fds=False)\nopen({:?}, 'w').write(str(child.pid))",
        pid_file.path()
    );
    let started = std::time::Instant::now();

    // A loaded Windows runner needs enough time to start Python before the
    // inherited-pipe deadline itself can be exercised.
    let result = run_fixture(&parent, INHERITED_PIPE_TIMEOUT).await;

    assert_eq!(result.unwrap_err().category(), "timeout");
    assert!(started.elapsed() < INHERITED_PIPE_TIMEOUT + Duration::from_millis(900));
    let pid = std::fs::read_to_string(pid_file.path())
        .expect("descendant pid")
        .parse::<u32>()
        .expect("numeric descendant pid");
    assert!(wait_until_process_is_gone(pid).await);
}

#[tokio::test]
async fn successful_parent_closes_inherited_pipes_without_consuming_the_deadline() {
    let _guard = LOG_GUARD.lock().await;
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let parent = format!(
        "import os,subprocess,sys\nstdout=os.dup(1)\nstderr=os.dup(2)\nos.set_inheritable(stdout, True)\nos.set_inheritable(stderr, True)\nchild=subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], stdout=stdout, stderr=stderr, close_fds=False)\nopen({:?}, 'w').write(str(child.pid))\nraise SystemExit(0)",
        pid_file.path()
    );
    let started = std::time::Instant::now();

    let result = run_fixture(&parent, Duration::from_secs(3)).await;

    assert!(result.is_ok());
    assert!(started.elapsed() < Duration::from_millis(2_900));
    let pid = std::fs::read_to_string(pid_file.path())
        .expect("descendant pid")
        .parse::<u32>()
        .expect("numeric descendant pid");
    assert!(wait_until_process_is_gone(pid).await);
}

#[cfg(unix)]
#[tokio::test]
async fn successful_parent_stays_successful_when_a_pipe_holder_escapes_its_group() {
    let _guard = LOG_GUARD.lock().await;
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let parent = format!(
        "import os,subprocess,sys\nstdout=os.dup(1)\nstderr=os.dup(2)\nos.set_inheritable(stdout, True)\nos.set_inheritable(stderr, True)\nchild=subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], stdout=stdout, stderr=stderr, pass_fds=(stdout, stderr), start_new_session=True)\nopen({:?}, 'w').write(str(child.pid))\nraise SystemExit(0)",
        pid_file.path()
    );

    let result = run_fixture(&parent, Duration::from_secs(3)).await;
    let pid = std::fs::read_to_string(pid_file.path())
        .expect("escaped descendant pid")
        .parse::<u32>()
        .expect("numeric descendant pid");
    unsafe { libc::kill(pid as i32, libc::SIGKILL) };

    assert!(result.is_ok());
}

#[tokio::test]
async fn cancellation_terminates_the_owned_process() {
    let _guard = LOG_GUARD.lock().await;
    let pid_file = tempfile::NamedTempFile::new().expect("pid file");
    let code = format!(
        "import os,time; open({:?}, 'w').write(str(os.getpid())); time.sleep(30)",
        pid_file.path()
    );
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let cancellation = admission.cancellation();
    let mut command = fixture_command(&code);
    let running = tokio::spawn(async move {
        run_runtime_command(
            &mut command,
            RuntimeStage::ValidateImports,
            tokio::time::Instant::now() + Duration::from_secs(1),
            &cancellation,
        )
        .await
    });
    wait_for_file(pid_file.path()).await;
    coordinator.close_work_admission_for_test();

    let result = running.await.expect("runtime task");
    assert_eq!(result.unwrap_err().category(), "cancelled");
    let pid = std::fs::read_to_string(pid_file.path())
        .expect("child pid")
        .parse::<u32>()
        .expect("numeric child pid");
    assert!(wait_until_process_is_gone(pid).await);
}

#[tokio::test]
async fn non_utf8_diagnostics_are_rendered_without_invalid_log_bytes() {
    let _guard = LOG_GUARD.lock().await;
    let result = run_fixture(
        "import sys; sys.stdout.buffer.write(b'\\xfftail'); sys.stderr.buffer.write(b'\\xfetail'); raise SystemExit(4)",
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let log = std::fs::read(super::paths::runtime_log_path()).expect("runtime log");
    assert!(std::str::from_utf8(&log).is_ok());
    assert!(log.len() <= 16_384);
}

#[tokio::test]
async fn diagnostics_redact_sensitive_markers_and_their_values() {
    let _guard = LOG_GUARD.lock().await;
    let protected_value = format!("value-{}", std::process::id());
    let marker = ["authori", "zation:"].concat();
    let scheme = ["bear", "er"].concat();
    let result = run_fixture(
        &format!("import sys; print('{marker} {scheme} {protected_value}', file=sys.stderr); raise SystemExit(5)"),
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let log = std::fs::read_to_string(super::paths::runtime_log_path()).expect("runtime log");
    assert!(!log.contains(&protected_value));
}

#[tokio::test]
async fn diagnostics_keep_redaction_across_a_separator() {
    let _guard = LOG_GUARD.lock().await;
    let protected_value = format!("protected-value-{}", std::process::id());
    let marker = ["pass", "word"].concat();
    let result = run_fixture(
        &format!(
            "import sys; print('{marker} = {protected_value}', file=sys.stderr); raise SystemExit(5)"
        ),
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let log = std::fs::read_to_string(super::paths::runtime_log_path()).expect("runtime log");
    assert!(!log.contains(&protected_value));
}

#[tokio::test]
async fn diagnostics_redact_every_supported_private_marker() {
    let _guard = LOG_GUARD.lock().await;
    let markers = [
        ["pass", "phrase"].concat(),
        ["pass", "wd"].concat(),
        ["private", "_key"].concat(),
        ["private", "-key"].concat(),
        ["coo", "kie"].concat(),
    ];
    for marker in markers {
        let protected = format!("private-value-{}", std::process::id());
        let result = run_fixture(
            &format!(
                "import sys; print('{marker} {protected}', file=sys.stderr); raise SystemExit(5)"
            ),
            Duration::from_secs(1),
        )
        .await;
        assert_eq!(result.unwrap_err().category(), "non-zero");
        let log = std::fs::read_to_string(super::paths::runtime_log_path()).expect("runtime log");
        assert!(
            !log.contains(&protected),
            "marker {marker} leaked its value"
        );
    }
}

#[tokio::test]
async fn multibyte_diagnostic_truncation_stays_valid_utf8() {
    let _guard = LOG_GUARD.lock().await;
    let result = run_fixture(
        "import sys; print('é' * 5000, file=sys.stderr); raise SystemExit(4)",
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let log = std::fs::read(super::paths::runtime_log_path()).expect("runtime log");
    assert!(std::str::from_utf8(&log).is_ok());
    assert!(log.len() <= 16_384);
}

#[tokio::test]
async fn second_diagnostic_replaces_the_first_even_with_a_stale_legacy_temp() {
    let _guard = LOG_GUARD.lock().await;
    let log = super::paths::runtime_log_path();
    std::fs::create_dir_all(log.parent().expect("log parent")).expect("log parent");
    let stale = log.with_extension("log.next");
    let _ = std::fs::remove_file(&stale);
    let first = run_fixture(
        "print('first-tail'); raise SystemExit(1)",
        Duration::from_secs(1),
    )
    .await;
    assert_eq!(first.unwrap_err().category(), "non-zero");
    std::fs::write(&stale, b"stale").expect("stale legacy temp");

    let second = run_fixture(
        "print('second-tail'); raise SystemExit(2)",
        Duration::from_secs(1),
    )
    .await;

    assert_eq!(second.unwrap_err().category(), "non-zero");
    let body = std::fs::read_to_string(&log).expect("second runtime log");
    assert!(body.contains("second-tail"));
    assert!(!body.contains("first-tail"));
    std::fs::remove_file(stale).expect("remove stale legacy temp");
}

#[tokio::test]
async fn oversized_legacy_diagnostic_is_replaced_instead_of_becoming_absorbing() {
    let _guard = LOG_GUARD.lock().await;
    let log = super::paths::runtime_log_path();
    std::fs::create_dir_all(log.parent().expect("log parent")).expect("log parent");
    std::fs::write(&log, vec![b'x'; 20_000]).expect("oversized legacy log");

    let result = run_fixture("raise SystemExit(3)", Duration::from_secs(1)).await;

    assert_eq!(result.unwrap_err().category(), "non-zero");
    let body = std::fs::read(&log).expect("replacement log");
    assert!(body.len() <= 16_384);
}

#[cfg(unix)]
#[tokio::test]
async fn diagnostics_refuse_a_symlinked_log_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let _guard = LOG_GUARD.lock().await;
    let log = super::paths::runtime_log_path();
    std::fs::create_dir_all(log.parent().expect("log parent")).expect("log parent");
    let saved = std::fs::read(&log).ok();
    let _ = std::fs::remove_file(&log);
    let outside = tempfile::NamedTempFile::new().expect("outside log target");
    std::fs::write(outside.path(), b"outside").expect("outside marker");
    symlink(outside.path(), &log).expect("log symlink");

    let result = run_fixture("raise SystemExit(1)", Duration::from_secs(1)).await;

    assert_eq!(result.unwrap_err().category(), "diagnostics");
    assert_eq!(
        std::fs::read(outside.path()).expect("outside body"),
        b"outside"
    );
    std::fs::remove_file(&log).expect("remove test symlink");
    if let Some(saved) = saved {
        std::fs::write(log, saved).expect("restore log");
    }
}

#[tokio::test]
async fn diagnostics_refuse_a_hard_linked_log_without_touching_its_target() {
    let _guard = LOG_GUARD.lock().await;
    let log = super::paths::runtime_log_path();
    std::fs::create_dir_all(log.parent().expect("log parent")).expect("log parent");
    let saved = std::fs::read(&log).ok();
    let _ = std::fs::remove_file(&log);
    let outside = tempfile::NamedTempFile::new().expect("outside log target");
    std::fs::write(outside.path(), b"outside").expect("outside marker");
    std::fs::hard_link(outside.path(), &log).expect("hard-linked log");

    let result = run_fixture("raise SystemExit(1)", Duration::from_secs(1)).await;

    assert_eq!(result.unwrap_err().category(), "diagnostics");
    assert_eq!(
        std::fs::read(outside.path()).expect("outside body"),
        b"outside"
    );
    std::fs::remove_file(&log).expect("remove test hard link");
    if let Some(saved) = saved {
        std::fs::write(log, saved).expect("restore log");
    }
}

async fn run_fixture(
    code: &str,
    timeout: Duration,
) -> Result<(), super::runtime_command::RuntimeCommandError> {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let mut command = fixture_command(code);
    run_runtime_command(
        &mut command,
        RuntimeStage::ValidateImports,
        tokio::time::Instant::now() + timeout,
        &admission.cancellation(),
    )
    .await
}

fn fixture_command(code: &str) -> Command {
    let python = crate::services::test_runtime::python().expect("runtime Python de test");
    let mut command = Command::new(python);
    command.args(["-c", code]);
    command
}

async fn wait_for_file(path: &std::path::Path) {
    for _ in 0..100 {
        if std::fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("child did not write pid");
}

async fn wait_until_process_is_gone(pid: u32) -> bool {
    for _ in 0..100 {
        let mut processes = sysinfo::System::new();
        processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if processes.process(sysinfo::Pid::from_u32(pid)).is_none() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    false
}
