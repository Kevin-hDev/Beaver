use super::operation_failure::OperationFailure;

#[derive(Clone, Copy)]
pub enum Operation {
    InstallGit,
    InstallNpm,
    Update,
    Uninstall,
    Cleanup,
}

impl Operation {
    fn label(self) -> &'static str {
        match self {
            Self::InstallGit => "install_git",
            Self::InstallNpm => "install_npm",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
            Self::Cleanup => "cleanup",
        }
    }
}

pub fn report(operation: Operation, failure: OperationFailure) -> String {
    let code = failure.code();
    super::operation_log::write(operation.label(), code, failure.reason());
    code.to_string()
}

pub(super) fn close(operation: &str, error: String) -> String {
    if super::error_codes::ALL.contains(&error.as_str()) {
        return error;
    }
    let code = super::error_codes::OPERATION_FAILED;
    let reason = "operation_failed";
    super::operation_log::write(operation, code, reason);
    code.to_string()
}

pub(super) fn is_safe_reason(reason: &str) -> bool {
    super::error_codes::ALL
        .iter()
        .filter_map(|code| code.strip_prefix("extensions_"))
        .any(|known| known == reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::extensions::error_codes;

    #[test]
    fn stable_failures_expose_specific_safe_codes() {
        assert_eq!(
            OperationFailure::GitTimeout.code(),
            error_codes::GIT_TIMEOUT
        );
        assert_eq!(
            OperationFailure::NotBeaverExtension.code(),
            error_codes::NOT_BEAVER_EXTENSION
        );
        assert_eq!(
            OperationFailure::ApiIncompatible.code(),
            error_codes::API_INCOMPATIBLE
        );
        assert_eq!(
            OperationFailure::from(
                crate::services::extensions::process_runner::ProcessFailure::EnvironmentInvalid
            ),
            OperationFailure::EnvironmentInvalid
        );
    }

    #[test]
    fn classified_command_failures_keep_specific_contract_codes() {
        assert_eq!(
            close(
                "set_enabled",
                error_codes::ACTIVATION_CONFIRMATION_REQUIRED.to_string()
            ),
            error_codes::ACTIVATION_CONFIRMATION_REQUIRED
        );
        assert_eq!(
            close("open_source", error_codes::NOT_FOUND.to_string()),
            error_codes::NOT_FOUND
        );
        assert_eq!(
            close("reload_host", error_codes::HOST_INCOMPATIBLE.to_string()),
            error_codes::HOST_INCOMPATIBLE
        );
    }

    #[test]
    fn legacy_sentences_cannot_become_a_second_public_error_authority() {
        for sentence in [
            "Confirmation d'activation requise.",
            "Extension introuvable.",
        ] {
            assert_eq!(
                close("legacy", sentence.to_string()),
                error_codes::OPERATION_FAILED
            );
        }
    }
}
