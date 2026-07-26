#![cfg(unix)]

use super::subagent_worktree_cleanup::{remove_managed_path_with_runner, GitRemoveRunner};
use std::future::Future;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};

struct DenyInspectionRunner {
    denied: AtomicBool,
}

impl GitRemoveRunner for DenyInspectionRunner {
    fn run<'a>(
        &'a self,
        path: &'a Path,
        _retry_locked: bool,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            if !self.denied.swap(true, Ordering::SeqCst) {
                let parent = path.parent().expect("managed child directory");
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o000))
                    .expect("deny inspection");
            }
            false
        })
    }
}

#[tokio::test]
async fn inspection_error_is_never_treated_as_missing() {
    let root = tempfile::tempdir().expect("managed root");
    let child_dir = root.path().join(uuid::Uuid::new_v4().to_string());
    let target = child_dir.join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir_all(&target)
        .await
        .expect("create target");
    tokio::fs::write(target.join(".git"), "must stay")
        .await
        .expect("create git marker");
    let canonical_root = tokio::fs::canonicalize(root.path())
        .await
        .expect("canonical root");
    let runner = DenyInspectionRunner {
        denied: AtomicBool::new(false),
    };

    let result = remove_managed_path_with_runner(&target, &canonical_root, &runner).await;
    std::fs::set_permissions(&child_dir, std::fs::Permissions::from_mode(0o700))
        .expect("restore inspection");

    assert!(result.is_err(), "une erreur d'inspection doit bloquer");
    assert!(target.exists(), "la cible doit rester intacte");
    assert!(target.join(".git").exists(), "les métadonnées Git doivent rester");
}
