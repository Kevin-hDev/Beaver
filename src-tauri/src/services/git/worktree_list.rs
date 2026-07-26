use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

const MAX_WORKTREES: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub is_current: bool,
}

pub async fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>, String> {
    let current_root = current_worktree_root(repo_path);
    let internal_root = crate::services::paths::data_dir().join("subagent-worktrees");
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Git indisponible".to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_worktrees(&stdout, &current_root, &internal_root))
}

fn parse_worktrees(stdout: &str, current_root: &Path, internal_root: &Path) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_branch = String::new();
    let mut is_bare = false;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = path.to_string();
            current_branch.clear();
            is_bare = false;
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = branch.to_string();
        } else if line == "bare" {
            is_bare = true;
        } else if line.is_empty() && !current_path.is_empty() {
            push_worktree(
                &mut worktrees,
                &current_path,
                &current_branch,
                is_bare,
                current_root,
                internal_root,
            );
            current_path.clear();
        }
    }

    if !current_path.is_empty() {
        push_worktree(
            &mut worktrees,
            &current_path,
            &current_branch,
            is_bare,
            current_root,
            internal_root,
        );
    }

    worktrees
}

fn push_worktree(
    worktrees: &mut Vec<WorktreeInfo>,
    path: &str,
    branch: &str,
    is_bare: bool,
    current_root: &Path,
    internal_root: &Path,
) {
    if is_bare || worktrees.len() >= MAX_WORKTREES {
        return;
    }

    let canonical_path = canonical_or_original(Path::new(path));
    if is_internal_worktree(&canonical_path, internal_root) {
        return;
    }

    let canonical_current = canonical_or_original(current_root);
    worktrees.push(WorktreeInfo {
        path: canonical_path.to_string_lossy().to_string(),
        branch: branch.to_string(),
        is_current: canonical_path == canonical_current,
    });
}

fn current_worktree_root(repo_path: &Path) -> PathBuf {
    git2::Repository::discover(repo_path)
        .ok()
        .and_then(|repo| repo.workdir().map(Path::to_path_buf))
        .map(|path| canonical_or_original(&path))
        .unwrap_or_else(|| canonical_or_original(repo_path))
}

fn canonical_or_original(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn is_internal_worktree(path: &Path, internal_root: &Path) -> bool {
    let root = canonical_or_original(internal_root);
    path.starts_with(&root)
}

#[cfg(test)]
mod tests {
    use super::parse_worktrees;
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("cl-go-worktrees-{name}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).expect("create test dir");
        path
    }

    fn porcelain(paths: &[(&Path, &str)]) -> String {
        paths
            .iter()
            .map(|(path, branch)| {
                format!(
                    "worktree {}\nHEAD 0000000000000000000000000000000000000000\nbranch refs/heads/{branch}\n\n",
                    path.display()
                )
            })
            .collect()
    }

    #[test]
    fn marks_current_worktree() {
        let current = temp_dir("current");
        let other = temp_dir("other");
        let internal = temp_dir("internal-root");
        let stdout = porcelain(&[(&current, "main"), (&other, "feature")]);

        let worktrees = parse_worktrees(&stdout, &current, &internal);

        assert_eq!(worktrees.len(), 2);
        assert!(worktrees.iter().any(|w| w.branch == "main" && w.is_current));
        assert!(worktrees
            .iter()
            .any(|w| w.branch == "feature" && !w.is_current));

        let _ = std::fs::remove_dir_all(current);
        let _ = std::fs::remove_dir_all(other);
        let _ = std::fs::remove_dir_all(internal);
    }

    #[test]
    fn excludes_internal_subagent_worktrees() {
        let current = temp_dir("current");
        let internal = temp_dir("internal-root");
        let child = internal.join("child-session");
        std::fs::create_dir_all(&child).expect("create child worktree");
        let stdout = porcelain(&[(&current, "main"), (&child, "detached-child")]);

        let worktrees = parse_worktrees(&stdout, &current, &internal);

        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].branch, "main");

        let _ = std::fs::remove_dir_all(current);
        let _ = std::fs::remove_dir_all(internal);
    }
}
