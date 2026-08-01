use super::types_tools::ToolResult;

pub(super) enum SpreadsheetReadError {
    Invalid(&'static str, String),
    NotFound(String),
    SourceInvalid(String),
    ReadFailed(String),
}

impl SpreadsheetReadError {
    pub(super) fn invalid(code: &'static str, message: impl Into<String>) -> Self {
        Self::Invalid(code, message.into())
    }

    pub(super) fn source(message: impl Into<String>) -> Self {
        Self::SourceInvalid(message.into())
    }

    pub(super) fn read(message: impl Into<String>) -> Self {
        Self::ReadFailed(message.into())
    }

    pub(super) fn message(&self) -> &str {
        match self {
            Self::Invalid(_, message)
            | Self::NotFound(message)
            | Self::SourceInvalid(message)
            | Self::ReadFailed(message) => message,
        }
    }

    pub(super) fn into_tool_result(self) -> ToolResult {
        match self {
            Self::Invalid(code, message) => ToolResult::validation(code, message),
            Self::NotFound(message) => ToolResult::not_found(
                "spreadsheet_sheet_not_found",
                message,
            )
            .with_error_hint(
                "Relancer la lecture sans préciser de feuille pour obtenir les feuilles disponibles.",
            ),
            Self::SourceInvalid(message) => {
                ToolResult::validation("spreadsheet_source_invalid", message)
            }
            Self::ReadFailed(message) => {
                ToolResult::execution("spreadsheet_read_failed", message, true)
            }
        }
    }
}

pub(super) enum SpreadsheetWriteError {
    Invalid(String),
    SourceInvalid(String),
    WriteFailed(String),
}

impl SpreadsheetWriteError {
    pub(super) fn source(message: impl Into<String>) -> Self {
        Self::SourceInvalid(message.into())
    }

    pub(super) fn write(message: impl Into<String>) -> Self {
        Self::WriteFailed(message.into())
    }

    pub(super) fn into_tool_result(self) -> ToolResult {
        match self {
            Self::Invalid(message) => {
                ToolResult::validation("spreadsheet_operation_invalid", message)
            }
            Self::SourceInvalid(message) => {
                ToolResult::validation("spreadsheet_source_invalid", message)
            }
            Self::WriteFailed(message) => ToolResult::execution(
                "spreadsheet_write_failed",
                message,
                false,
            )
            .with_error_hint(
                "Vérifier le classeur cible avant toute nouvelle écriture : il peut être partiel.",
            ),
        }
    }
}

impl From<String> for SpreadsheetWriteError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::tool_result_contract::ToolErrorCategory;

    #[test]
    fn read_failures_keep_distinct_recovery_rules() {
        let missing = SpreadsheetReadError::NotFound("missing".into()).into_tool_result();
        let transient = SpreadsheetReadError::read("unavailable").into_tool_result();

        assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
        assert!(transient.error.unwrap().retryable);
    }

    #[test]
    fn write_input_is_not_reported_as_an_uncertain_write() {
        let invalid = SpreadsheetWriteError::Invalid("bad operation".into()).into_tool_result();
        let failed = SpreadsheetWriteError::write("save failed").into_tool_result();

        assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
        assert!(!failed.error.unwrap().retryable);
    }
}
