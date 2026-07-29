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
}
