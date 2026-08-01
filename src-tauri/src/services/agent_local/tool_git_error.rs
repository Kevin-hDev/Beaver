use crate::services::git::action_error::GitActionError;
use crate::services::git::branch::CreateBranchError;

use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(super) fn create_branch(error: CreateBranchError) -> ToolResult {
    match error {
        CreateBranchError::InvalidName => failure(
            "Nom de branche invalide.",
            "invalid_branch_name",
            ToolErrorCategory::Validation,
        ),
        CreateBranchError::NameTooLong => failure(
            "Nom de branche trop long.",
            "branch_name_too_long",
            ToolErrorCategory::Validation,
        ),
        CreateBranchError::AlreadyExists => failure(
            "Cette branche existe déjà.",
            "branch_already_exists",
            ToolErrorCategory::Conflict,
        ),
        CreateBranchError::UnbornHead => failure(
            "Le dépôt ne contient encore aucun commit.",
            "repository_has_no_commit",
            ToolErrorCategory::Conflict,
        )
        .with_error_hint("Créer le premier commit avant de créer une branche."),
        CreateBranchError::GithubAuthRequired => failure(
            "Authentification GitHub requise.",
            "git_authentication_required",
            ToolErrorCategory::Permission,
        ),
        CreateBranchError::InternalError => uncertain(
            "La création de la branche n'a pas pu être confirmée.",
            "branch_creation_failed",
            ToolErrorCategory::Internal,
        ),
    }
}

pub(super) fn checkout_branch(error: GitActionError) -> ToolResult {
    match error {
        GitActionError::RepositoryUnavailable => failure(
            "Dépôt Git indisponible.",
            "git_repository_unavailable",
            ToolErrorCategory::NotFound,
        ),
        GitActionError::BranchUnavailable => failure(
            "Branche Git introuvable.",
            "git_branch_not_found",
            ToolErrorCategory::NotFound,
        ),
        GitActionError::DirtyWorktree { dirty_count } => failure(
            format!("Le dépôt contient {dirty_count} changement(s) non enregistré(s)."),
            "git_worktree_dirty",
            ToolErrorCategory::Conflict,
        )
        .with_error_hint("Examiner et préserver ces changements avant de changer de branche."),
        GitActionError::CheckoutFailed | GitActionError::InternalError => uncertain(
            "Le changement de branche n'a pas pu être confirmé.",
            "git_checkout_failed",
            ToolErrorCategory::Internal,
        ),
        GitActionError::ProtectedBranch
        | GitActionError::BranchActive
        | GitActionError::NoFallbackBranch
        | GitActionError::UnmergedCommits { .. } => failure(
            "L'état du dépôt empêche ce changement de branche.",
            "git_state_conflict",
            ToolErrorCategory::Conflict,
        ),
        GitActionError::IdentityMissing | GitActionError::InvalidCommitDescription => failure(
            "Configuration Git invalide.",
            "git_configuration_invalid",
            ToolErrorCategory::Validation,
        ),
        GitActionError::CommitFailed
        | GitActionError::MergeFailed
        | GitActionError::DeleteFailed
        | GitActionError::WorktreeUnavailable
        | GitActionError::CloneUnavailable => uncertain(
            "L'opération Git n'a pas pu être confirmée.",
            "git_action_failed",
            ToolErrorCategory::Execution,
        ),
    }
}

fn failure(
    message: impl Into<String>,
    code: &'static str,
    category: ToolErrorCategory,
) -> ToolResult {
    ToolResult::error(message, code, category, false)
}

fn uncertain(message: &'static str, code: &'static str, category: ToolErrorCategory) -> ToolResult {
    failure(message, code, category).with_error_hint(
        "Vérifier la branche active et l'état du dépôt avant toute nouvelle tentative.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_causes_keep_distinct_codes() {
        let invalid = create_branch(CreateBranchError::InvalidName);
        let exists = create_branch(CreateBranchError::AlreadyExists);
        let auth = create_branch(CreateBranchError::GithubAuthRequired);

        assert_eq!(invalid.error.unwrap().code.as_ref(), "invalid_branch_name");
        assert_eq!(exists.error.unwrap().category, ToolErrorCategory::Conflict);
        assert_eq!(auth.error.unwrap().category, ToolErrorCategory::Permission);
    }

    #[test]
    fn uncertain_creation_requires_state_verification() {
        let result = create_branch(CreateBranchError::InternalError);
        let error = result.error.unwrap();

        assert!(!error.retryable);
        assert!(error.hint.is_some());
    }

    #[test]
    fn checkout_distinguishes_dirty_missing_and_uncertain_states() {
        let dirty = checkout_branch(GitActionError::DirtyWorktree { dirty_count: 2 });
        let missing = checkout_branch(GitActionError::BranchUnavailable);
        let uncertain = checkout_branch(GitActionError::CheckoutFailed);

        assert_eq!(dirty.error.unwrap().code.as_ref(), "git_worktree_dirty");
        assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
        let uncertain_error = uncertain.error.unwrap();
        assert!(!uncertain_error.retryable);
        assert!(uncertain_error.hint.is_some());
    }
}
