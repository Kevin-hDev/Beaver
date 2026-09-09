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
            // A setting rejected before transport is not a failed reasoning replay.
            Self::ReasoningConfigurationInvalid => "reasoning_configuration_invalid",
            Self::ModelInvalid => "model_not_found",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChatTargetError;

    #[test]
    fn reasoning_setting_error_does_not_claim_replay_failure() {
        let error = ChatTargetError::ReasoningConfigurationInvalid;
        assert_eq!(error.ui_code(), "reasoning_configuration_invalid");
        assert_eq!(error.ui_code(), error.diagnostic_code());
        assert_ne!(error.ui_code(), "reasoning_continuity_invalid");
    }
}
