#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatTargetError {
    SessionInconsistent,
    CatalogUnavailable,
    ReasoningConfigurationInvalid,
    ModelInvalid,
}

impl ChatTargetError {
    pub(crate) const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::SessionInconsistent => "session_inconsistent",
            Self::CatalogUnavailable => "model_catalog_unavailable",
            Self::ReasoningConfigurationInvalid => "reasoning_configuration_invalid",
            Self::ModelInvalid => "model_invalid",
        }
    }

    pub(crate) const fn ui_code(self) -> &'static str {
        match self {
            Self::SessionInconsistent => "conversation_admission_failed",
            Self::CatalogUnavailable => "model_catalog_unavailable",
            Self::ReasoningConfigurationInvalid => "reasoning_continuity_invalid",
            Self::ModelInvalid => "model_not_found",
        }
    }
}
