use super::error_codes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationFailure {
    InstallFailed,
    UpdateFailed,
    UninstallFailed,
    SourceInvalid,
    PackageInvalid,
    GitDownloadFailed,
    GitTimeout,
    RuntimeUnavailable,
    EnvironmentInvalid,
    DependencyInstallFailed,
    ManifestInvalid,
    NotBeaverExtension,
    ApiIncompatible,
    SymlinkUnsupported,
    AlreadyInstalled,
    LimitReached,
    StorageFailed,
    UpdateIdentityChanged,
    UpdateUnavailable,
    CleanupFailed,
}

impl OperationFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::InstallFailed => error_codes::INSTALL_FAILED,
            Self::UpdateFailed => error_codes::UPDATE_FAILED,
            Self::UninstallFailed => error_codes::UNINSTALL_FAILED,
            Self::SourceInvalid => error_codes::SOURCE_INVALID,
            Self::PackageInvalid => error_codes::PACKAGE_INVALID,
            Self::GitDownloadFailed => error_codes::GIT_DOWNLOAD_FAILED,
            Self::GitTimeout => error_codes::GIT_TIMEOUT,
            Self::RuntimeUnavailable => error_codes::RUNTIME_UNAVAILABLE,
            Self::EnvironmentInvalid => error_codes::ENVIRONMENT_INVALID,
            Self::DependencyInstallFailed => error_codes::DEPENDENCY_INSTALL_FAILED,
            Self::ManifestInvalid => error_codes::MANIFEST_INVALID,
            Self::NotBeaverExtension => error_codes::NOT_BEAVER_EXTENSION,
            Self::ApiIncompatible => error_codes::API_INCOMPATIBLE,
            Self::SymlinkUnsupported => error_codes::SYMLINK_UNSUPPORTED,
            Self::AlreadyInstalled => error_codes::ALREADY_INSTALLED,
            Self::LimitReached => error_codes::LIMIT_REACHED,
            Self::StorageFailed => error_codes::STORAGE_FAILED,
            Self::UpdateIdentityChanged => error_codes::UPDATE_IDENTITY_CHANGED,
            Self::UpdateUnavailable => error_codes::UPDATE_UNAVAILABLE,
            Self::CleanupFailed => error_codes::CLEANUP_FAILED,
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::InstallFailed => "install_failed",
            Self::UpdateFailed => "update_failed",
            Self::UninstallFailed => "uninstall_failed",
            Self::SourceInvalid => "source_invalid",
            Self::PackageInvalid => "package_invalid",
            Self::GitDownloadFailed => "git_download_failed",
            Self::GitTimeout => "git_timeout",
            Self::RuntimeUnavailable => "runtime_unavailable",
            Self::EnvironmentInvalid => "environment_invalid",
            Self::DependencyInstallFailed => "dependency_install_failed",
            Self::ManifestInvalid => "manifest_invalid",
            Self::NotBeaverExtension => "not_beaver_extension",
            Self::ApiIncompatible => "api_incompatible",
            Self::SymlinkUnsupported => "symlink_unsupported",
            Self::AlreadyInstalled => "already_installed",
            Self::LimitReached => "limit_reached",
            Self::StorageFailed => "storage_failed",
            Self::UpdateIdentityChanged => "update_identity_changed",
            Self::UpdateUnavailable => "update_unavailable",
            Self::CleanupFailed => "cleanup_failed",
        }
    }
}

impl From<super::process_runner::ProcessFailure> for OperationFailure {
    fn from(failure: super::process_runner::ProcessFailure) -> Self {
        match failure {
            super::process_runner::ProcessFailure::EnvironmentInvalid => Self::EnvironmentInvalid,
            super::process_runner::ProcessFailure::CommandInvalid
            | super::process_runner::ProcessFailure::Unavailable => Self::RuntimeUnavailable,
            super::process_runner::ProcessFailure::Failed
            | super::process_runner::ProcessFailure::Timeout
            | super::process_runner::ProcessFailure::Interrupted => Self::DependencyInstallFailed,
        }
    }
}
