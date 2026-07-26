use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GitActionError {
    #[error("repository unavailable")]
    RepositoryUnavailable,
    #[error("branch unavailable")]
    BranchUnavailable,
    #[error("working tree contains changes")]
    DirtyWorktree { dirty_count: usize },
    #[error("no fallback branch available")]
    NoFallbackBranch,
    #[error("protected branch")]
    ProtectedBranch,
    #[error("branch is active")]
    BranchActive,
    #[error("git identity unavailable")]
    IdentityMissing,
    #[error("invalid commit description")]
    InvalidCommitDescription,
    #[error("branch checkout failed")]
    CheckoutFailed,
    #[error("commit failed")]
    CommitFailed,
    #[error("merge failed")]
    MergeFailed,
    #[error("branch contains unmerged commits")]
    UnmergedCommits { count: usize },
    #[error("branch or worktree deletion failed")]
    DeleteFailed,
    #[error("worktree unavailable")]
    WorktreeUnavailable,
    #[error("clone unavailable")]
    CloneUnavailable,
    #[error("internal git error")]
    InternalError,
}
