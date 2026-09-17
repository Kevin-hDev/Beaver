use super::types_tools::ToolResult;

pub(super) fn file_error(error: crate::services::extensions::FileReadError) -> ToolResult {
    match error {
        crate::services::extensions::FileReadError::Cancelled => ToolResult::cancelled("Annulé."),
        crate::services::extensions::FileReadError::Limit => too_large_result(),
        _ => invalid_result(),
    }
}

pub(super) fn artifact_error(error: crate::services::extensions::FileResultError) -> ToolResult {
    match error {
        crate::services::extensions::FileResultError::Limit => too_large_result(),
        _ => invalid_result(),
    }
}

pub(super) fn invalid_result() -> ToolResult {
    ToolResult::unavailable(
        crate::services::extensions::error_codes::RESULT_INVALID,
        "Résultat d'extension indisponible.",
        false,
    )
}

pub(super) fn too_large_result() -> ToolResult {
    ToolResult::unavailable(
        crate::services::extensions::error_codes::RESULT_TOO_LARGE,
        "Résultat d'extension indisponible.",
        false,
    )
}
