use super::memory_store::{MemoryEditError, MemoryWriteError};
use super::types_tools::ToolResult;

pub(super) fn edit_error(error: MemoryEditError) -> ToolResult {
    match error {
        MemoryEditError::Stale => ToolResult::conflict(
            "memory_edit_stale",
            "Le sujet mémoire a changé. Relisez-le avant de le modifier.",
        )
        .with_error_hint("Relire le sujet mémoire puis recalculer la modification."),
        MemoryEditError::NotFound => {
            ToolResult::not_found("memory_topic_not_found", "Sujet mémoire introuvable.")
        }
        MemoryEditError::Failed(error) => mutation_error(error),
    }
}

pub(super) fn mutation_error(error: MemoryWriteError) -> ToolResult {
    match error {
        MemoryWriteError::SetupUnavailable(message) => {
            ToolResult::unavailable("memory_setup_unavailable", message, true)
        }
        MemoryWriteError::TargetInvalid(message) => {
            ToolResult::permission("memory_target_invalid", message)
        }
        MemoryWriteError::LimitReached => ToolResult::conflict(
            "memory_topic_limit_reached",
            "La limite de sujets mémoire est atteinte.",
        ),
        MemoryWriteError::ContentInvalid(message) => {
            ToolResult::validation("memory_content_invalid", message)
        }
        MemoryWriteError::SourceUnavailable(message) => {
            ToolResult::unavailable("memory_source_unavailable", message, true)
        }
        MemoryWriteError::StorageFailed(message) => {
            ToolResult::execution("memory_write_failed", message, false).with_error_hint(
                "Relire la mémoire concernée avant une nouvelle écriture : son état peut avoir changé.",
            )
        }
        MemoryWriteError::AppliedButIndexFailed(message) => ToolResult::partial(
            "Mémoire enregistrée, mais son index n'a pas pu être mis à jour.",
            [
                message,
                "Le sujet est déjà enregistré : ne pas répéter l'écriture avant de le relire."
                    .to_string(),
            ],
        ),
    }
}
