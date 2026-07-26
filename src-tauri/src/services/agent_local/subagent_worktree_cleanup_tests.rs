use super::{
    subagent_worktree,
    subagent_worktree_cleanup::{
        remove_managed_path_with_runner, GitRemoveRunner, GIT_REMOVE_TIMEOUT,
    },
    subagent_worktree_ownership_tests::init_repo_with_commit,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::path::Path;
use std::process::Command;

struct BlockingRunner {
    attempts: Arc<AtomicUsize>,
    abandoned: Arc<AtomicUsize>,
    retried_locked: Arc<AtomicBool>,
}

impl GitRemoveRunner for BlockingRunner {
    fn run<'a>(
        &'a self,
        _path: &'a Path,
        retry_locked: bool,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        self.retried_locked
            .fetch_or(retry_locked, Ordering::SeqCst);
        Box::pin(BlockingProcess {
            abandoned: Arc::clone(&self.abandoned),
        })
    }
}

struct BlockingProcess {
    abandoned: Arc<AtomicUsize>,
}

impl Future for BlockingProcess {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for BlockingProcess {
    fn drop(&mut self) {
        self.abandoned.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test(start_paused = true)]
async fn blocked_git_remove_is_abandoned_bounded_and_fail_closed() {
    let root = tempfile::tempdir().expect("managed root");
    let target = root.path().join(id()).join(id());
    tokio::fs::create_dir_all(&target)
        .await
        .expect("create sensitive target");
    tokio::fs::write(target.join(".git"), "must stay")
        .await
        .expect("create git marker");
    let canonical_root = tokio::fs::canonicalize(root.path())
        .await
        .expect("canonical managed root");
    let attempts = Arc::new(AtomicUsize::new(0));
    let abandoned = Arc::new(AtomicUsize::new(0));
    let retried_locked = Arc::new(AtomicBool::new(false));
    let runner = BlockingRunner {
        attempts: Arc::clone(&attempts),
        abandoned: Arc::clone(&abandoned),
        retried_locked: Arc::clone(&retried_locked),
    };
    let started = tokio::time::Instant::now();

    let result = tokio::time::timeout(
        GIT_REMOVE_TIMEOUT * 3,
        remove_managed_path_with_runner(&target, &canonical_root, &runner),
    )
    .await
    .expect("cleanup must finish within its bounded recovery window");

    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(abandoned.load(Ordering::SeqCst), 2);
    assert!(retried_locked.load(Ordering::SeqCst));
    assert_eq!(started.elapsed(), GIT_REMOVE_TIMEOUT * 2);
    assert!(target.exists());
    assert!(target.join(".git").exists());
}

#[tokio::test]
async fn locked_worktree_cleanup_removes_git_metadata() {
    let repo = init_repo_with_commit();
    let child_id = id();
    let execution_id = id();
    let target = subagent_worktree::create_for_execution(repo.path(), &child_id, &execution_id)
        .await
        .expect("create worktree");
    let locked = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["worktree", "lock"])
        .arg(&target)
        .status()
        .expect("lock worktree");
    assert!(locked.success());

    let result = subagent_worktree::remove(&target.to_string_lossy()).await;
    let listed = worktree_list(repo.path());

    assert!(result.is_ok());
    assert!(!target.exists());
    assert!(
        !listed.contains(target.to_string_lossy().as_ref()),
        "les métadonnées Git du worktree verrouillé subsistent"
    );
}

#[tokio::test]
async fn successful_cleanup_removes_empty_child_directory() {
    let repo = init_repo_with_commit();
    let target = subagent_worktree::create_for_execution(repo.path(), &id(), &id())
        .await
        .expect("create worktree");
    let child_dir = target.parent().expect("child directory").to_path_buf();

    subagent_worktree::remove(&target.to_string_lossy())
        .await
        .expect("remove worktree");

    assert!(!target.exists());
    assert!(!child_dir.exists(), "le répertoire UUID vide subsiste");
}

#[tokio::test]
async fn missing_execution_cleanup_removes_empty_child_directory() {
    let target = subagent_worktree::path_for_execution(&id(), &id()).expect("managed path");
    let child_dir = target.parent().expect("child directory").to_path_buf();
    tokio::fs::create_dir_all(&child_dir)
        .await
        .expect("create empty child directory");

    subagent_worktree::remove(&target.to_string_lossy())
        .await
        .expect("missing worktree is already removed");

    assert!(!child_dir.exists(), "le répertoire UUID vide subsiste après l'échec initial");
}

#[tokio::test]
async fn cleanup_never_removes_a_sibling_execution() {
    let repo = init_repo_with_commit();
    let child_id = id();
    let first = subagent_worktree::create_for_execution(repo.path(), &child_id, &id())
        .await
        .expect("create first worktree");
    let sibling = subagent_worktree::create_for_execution(repo.path(), &child_id, &id())
        .await
        .expect("create sibling worktree");
    let child_dir = first.parent().expect("child directory").to_path_buf();

    subagent_worktree::remove(&first.to_string_lossy())
        .await
        .expect("remove first worktree");

    assert!(child_dir.exists());
    assert!(sibling.exists(), "un autre run a été supprimé");
    subagent_worktree::remove(&sibling.to_string_lossy())
        .await
        .expect("remove sibling worktree");
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn worktree_list(repo: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .expect("list worktrees");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("utf8 worktree list")
}
