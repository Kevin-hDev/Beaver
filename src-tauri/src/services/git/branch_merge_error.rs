use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MergeError {
    #[error("branch unavailable")]
    BranchUnavailable,
    #[error("git context changed")]
    ContextChanged,
    #[error("working tree contains changes")]
    DirtyWorktree,
    #[error("nothing to merge")]
    NothingToMerge,
    #[error("merge conflict")]
    MergeConflict,
    #[error("internal git error")]
    InternalError,
}
