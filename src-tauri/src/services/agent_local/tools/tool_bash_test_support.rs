use super::*;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) async fn managed(
    command: &str,
    working_dir: &Path,
    owner: &str,
    timeout: Option<u64>,
    yield_time_ms: Option<u64>,
    cancel: CancellationToken,
) -> Result<crate::services::agent_local::types_tools::ShellOutput, String> {
    execute_shell_managed(
        command,
        working_dir,
        ShellExecutionContext {
            owner_session_id: owner,
            hard_timeout_secs: timeout,
            yield_time_ms,
            cancel,
            progress: None,
            work: crate::services::agent_local::agent_work_supervision::ShellWork::new(
                crate::app_exit::AppExitCoordinator::initialize()
                    .expect("exit coordinator")
                    .work_supervisor(),
            ),
        },
    )
    .await
}

pub(super) fn process_id(output: &str) -> &str {
    output
        .split("session_id=")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("process id")
}

pub(super) fn canonical(path: &Path) -> String {
    path.canonicalize()
        .expect("canonical path")
        .to_string_lossy()
        .to_string()
}

pub(super) fn commit_all(repository: &git2::Repository) {
    let mut index = repository.index().expect("index");
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree id");
    let tree = repository.find_tree(tree_id).expect("tree");
    let signature = git2::Signature::now("Beaver", "beaver@example.test").expect("signature");
    repository
        .commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])
        .expect("commit");
}

#[cfg(unix)]
pub(super) fn process_exists(pid: i32) -> bool {
    // SAFETY: signal 0 performs a read-only existence check for the captured child PID.
    let result = unsafe { libc::kill(pid, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}
