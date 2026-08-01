use super::security;
use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(super) fn io_failure(error: std::io::Error, fallback_code: &'static str) -> ToolResult {
    let uncertain_write = matches!(
        fallback_code,
        "file_write_failed" | "directory_create_failed"
    );
    let (code, category, retryable) = match error.kind() {
        std::io::ErrorKind::NotFound => {
            ("file_not_found", ToolErrorCategory::NotFound, false)
        }
        std::io::ErrorKind::PermissionDenied => {
            ("file_permission_denied", ToolErrorCategory::Permission, false)
        }
        std::io::ErrorKind::IsADirectory => (
            "path_is_directory",
            ToolErrorCategory::Validation,
            false,
        ),
        std::io::ErrorKind::NotADirectory => (
            "path_not_directory",
            ToolErrorCategory::Validation,
            false,
        ),
        std::io::ErrorKind::InvalidData if fallback_code == "file_read_failed" => {
            ("file_not_utf8", ToolErrorCategory::Validation, false)
        }
        std::io::ErrorKind::AlreadyExists => (
            "file_already_exists",
            ToolErrorCategory::Conflict,
            false,
        ),
        std::io::ErrorKind::TimedOut => (
            "file_io_timeout",
            ToolErrorCategory::Timeout,
            !uncertain_write,
        ),
        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock => {
            (fallback_code, ToolErrorCategory::Execution, !uncertain_write)
        }
        _ => (fallback_code, ToolErrorCategory::Execution, false),
    };
    let result = ToolResult::error(security::sanitize_error(error), code, category, retryable);
    if uncertain_write && matches!(category, ToolErrorCategory::Execution | ToolErrorCategory::Timeout)
    {
        result.with_error_hint(
            "Vérifier le fichier cible et son dossier avant toute nouvelle écriture : l'état peut être partiel.",
        )
    } else {
        result
    }
}

pub(super) fn directory_failure(error: std::io::Error) -> ToolResult {
    let (code, category, retryable) = match error.kind() {
        std::io::ErrorKind::NotFound => {
            ("directory_not_found", ToolErrorCategory::NotFound, false)
        }
        std::io::ErrorKind::PermissionDenied => (
            "directory_permission_denied",
            ToolErrorCategory::Permission,
            false,
        ),
        std::io::ErrorKind::NotADirectory => (
            "path_not_directory",
            ToolErrorCategory::Validation,
            false,
        ),
        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock => (
            "directory_read_failed",
            ToolErrorCategory::Execution,
            true,
        ),
        _ => (
            "directory_read_failed",
            ToolErrorCategory::Execution,
            false,
        ),
    };
    ToolResult::error(security::sanitize_error(error), code, category, retryable)
}

pub(super) fn path_failure(
    message: String,
    not_found_code: &'static str,
    denied_code: &'static str,
    invalid_code: &'static str,
) -> ToolResult {
    let lower = message.to_lowercase();
    let (code, category) = if lower.contains("introuvable")
        || lower.contains("not found")
        || lower.contains("no such file")
    {
        (not_found_code, ToolErrorCategory::NotFound)
    } else if lower.contains("interdit")
        || lower.contains("refus")
        || lower.contains("permission")
    {
        (denied_code, ToolErrorCategory::Permission)
    } else {
        (invalid_code, ToolErrorCategory::Validation)
    };
    ToolResult::error(message, code, category, false)
}

#[cfg(test)]
#[path = "tool_file_error_tests.rs"]
mod tests;
