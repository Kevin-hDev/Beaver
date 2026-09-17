use super::types_tools::ToolResult;
use super::tool_result_contract::ToolErrorCategory;
use serde_json::{json, Value};
use std::path::Path;

const APPLY_ERROR: &str = "Application du changement sous-agent impossible. Le changement isolé \
reste non résolu. Inspectez son état. Après une intégration manuelle, appelez \
discard_subagent_changes pour nettoyer le changement et sa branche temporaire.";

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    parent_id: &str,
) -> Option<ToolResult> {
    if !matches!(
        tool_name,
        "inspect_subagent_changes" | "apply_subagent_changes" | "discard_subagent_changes"
    ) {
        return None;
    }
    let child_id = match id_arg(args, "subagent_id") {
        Ok(value) => value,
        Err(result) => return Some(result),
    };
    let change_id = match id_arg(args, "change_id") {
        Ok(value) => value,
        Err(result) => return Some(result),
    };
    let result = match tool_name {
        "inspect_subagent_changes" => {
            match super::subagent_git_actions::inspect(
                working_dir,
                parent_id,
                child_id,
                change_id,
            )
            .await
            {
                Ok((change, patch, truncated)) => {
                    let mut result = ToolResult::ok(json!({
                        "change": change,
                        "patch": patch,
                        "patch_truncated": truncated
                    })
                    .to_string());
                    result.mark_truncated(truncated);
                    result
                }
                Err(error) => change_failure(
                    error,
                    "Inspection du changement sous-agent impossible.",
                    true,
                ),
            }
        }
        "apply_subagent_changes" => action_result(
            super::subagent_git_actions::apply(working_dir, parent_id, child_id, change_id).await,
            APPLY_ERROR,
        ),
        "discard_subagent_changes" => action_result(
            super::subagent_git_actions::discard(working_dir, parent_id, child_id, change_id).await,
            "Abandon du changement sous-agent impossible.",
        ),
        _ => unreachable!(),
    };
    Some(result)
}

fn action_result(
    result: Result<super::types_subagent_change::SubagentChangeMeta, String>,
    error: &str,
) -> ToolResult {
    match result {
        Ok(change) => {
            let paths = change
                .changed_paths
                .iter()
                .map(|changed| changed.path.clone())
                .collect();
            ToolResult::ok(json!({ "change": change }).to_string()).with_affected_paths(paths)
        }
        Err(cause) => change_failure(cause, error, false),
    }
}

fn id_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolResult> {
    let value = args[key].as_str().ok_or_else(invalid_id)?;
    super::types_subagent_change::validate_uuid(value).map_err(|_| invalid_id())?;
    Ok(value)
}

fn unavailable() -> ToolResult {
    ToolResult::error(
        "Changement sous-agent indisponible.",
        "subagent_change_unavailable",
        ToolErrorCategory::NotFound,
        false,
    )
}

fn invalid_id() -> ToolResult {
    ToolResult::error(
        "Identifiant de changement sous-agent invalide.",
        "invalid_subagent_change_id",
        ToolErrorCategory::Validation,
        false,
    )
}

fn change_failure(cause: String, fallback: &str, read_only: bool) -> ToolResult {
    let lower = cause.to_lowercase();
    if lower.contains("conflit") {
        return ToolResult::error(
            fallback,
            "subagent_change_conflict",
            ToolErrorCategory::Conflict,
            false,
        )
        .with_error_hint("Inspecter le changement et résoudre le conflit avant de poursuivre.");
    }
    if lower.contains("branche cible incompatible") {
        return ToolResult::error(
            fallback,
            "subagent_target_branch_changed",
            ToolErrorCategory::Conflict,
            false,
        )
        .with_error_hint("Revenir sur la branche cible du changement ou recréer celui-ci.");
    }
    if lower.contains("non prêt") || lower.contains("non abandonnable") {
        return ToolResult::error(
            fallback,
            "subagent_change_state_conflict",
            ToolErrorCategory::Conflict,
            false,
        )
        .with_error_hint("Inspecter l'état du dépôt parent et du changement avant de réessayer.");
    }
    if lower.contains("changement sous-agent indisponible")
        || lower.contains("projet sous-agent indisponible")
    {
        return unavailable();
    }
    if lower.contains("restauration") || lower.contains("persistance") {
        return ToolResult::error(
            fallback,
            "subagent_change_recovery_failed",
            ToolErrorCategory::Internal,
            false,
        )
        .with_error_hint("Inspecter manuellement le dépôt parent avant toute nouvelle opération Git.");
    }
    if lower.contains("indisponible") {
        let result = ToolResult::error(
            fallback,
            "subagent_change_dependency_unavailable",
            ToolErrorCategory::Unavailable,
            read_only,
        );
        return if read_only {
            result
        } else {
            result.with_error_hint(
                "Inspecter le dépôt parent avant de relancer : l'opération Git a pu être partiellement appliquée.",
            )
        };
    }
    if lower.contains("trop de projets") || lower.contains("limite") {
        return ToolResult::error(
            fallback,
            "subagent_change_capacity_reached",
            ToolErrorCategory::Conflict,
            true,
        );
    }
    ToolResult::error(
        fallback,
        "subagent_change_action_failed",
        ToolErrorCategory::Execution,
        false,
    )
    .with_error_hint(
        "Inspecter le dépôt parent et le changement avant toute nouvelle opération Git.",
    )
}

#[cfg(test)]
#[path = "tool_subagent_changes_tests.rs"]
mod tests;
