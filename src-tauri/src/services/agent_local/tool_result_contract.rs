use serde::{Deserialize, Serialize};

const MAX_HINT_CHARS: usize = 1_000;

#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResultStatus {
    #[default]
    Success,
    Running,
    Partial,
    Error,
    Cancelled,
    Stopped,
}

impl ToolResultStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Running => "running",
            Self::Partial => "partial",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::Stopped => "stopped",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCategory {
    Validation,
    Permission,
    NotFound,
    Conflict,
    Timeout,
    Cancelled,
    Unavailable,
    External,
    #[default]
    Execution,
    Internal,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolErrorInfo {
    pub code: Box<str>,
    pub category: ToolErrorCategory,
    /// True seulement si une nouvelle tentative est sûre sans vérifier l'état externe.
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Box<str>>,
}

impl ToolErrorInfo {
    pub fn new(code: &'static str, category: ToolErrorCategory, retryable: bool) -> Self {
        Self {
            code: code.into(),
            category,
            retryable,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(bound_safe_text(hint.into(), MAX_HINT_CHARS).into_boxed_str());
        self
    }
}

pub(super) fn bound_safe_text(input: String, max_chars: usize) -> String {
    input
        .chars()
        .filter(|character| safe_metadata_character(*character))
        .take(max_chars)
        .collect()
}

pub(super) fn is_safe_metadata_text(input: &str, max_chars: usize) -> bool {
    input.chars().count() <= max_chars && input.chars().all(safe_metadata_character)
}

pub(super) fn safe_metadata_character(character: char) -> bool {
    (!character.is_control() || matches!(character, '\n' | '\t'))
        && !matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'
                | '\u{2066}'..='\u{2069}'
                | '\u{feff}'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_have_unambiguous_error_semantics() {
        assert!(!ToolResultStatus::Success.is_error());
        assert!(!ToolResultStatus::Running.is_error());
        assert!(!ToolResultStatus::Partial.is_error());
        assert!(ToolResultStatus::Error.is_error());
        assert!(ToolResultStatus::Cancelled.is_error());
        assert!(!ToolResultStatus::Stopped.is_error());
    }

    #[test]
    fn hints_are_bounded_without_breaking_utf8() {
        let info = ToolErrorInfo::new("failure", ToolErrorCategory::Execution, false)
            .with_hint("🙂".repeat(MAX_HINT_CHARS + 1));

        assert_eq!(info.hint.as_deref().unwrap().chars().count(), MAX_HINT_CHARS);
    }

    #[test]
    fn hints_remove_unsafe_controls() {
        let info = ToolErrorInfo::new("failure", ToolErrorCategory::Execution, false)
            .with_hint("safe\u{202e}text\0");

        assert_eq!(info.hint.as_deref(), Some("safetext"));
        assert!(!is_safe_metadata_text("safe\u{202e}text", 100));
    }
}
