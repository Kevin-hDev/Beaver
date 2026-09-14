use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InstallerError {
    #[error("installer-invalid-launch")]
    InvalidLaunch,
    #[error("installer-download-failed")]
    DownloadFailed,
    #[error("installer-integrity-failed")]
    IntegrityFailed,
    #[error("installer-cleanup-failed")]
    CleanupFailed,
    #[error("installer-install-failed")]
    InstallFailed,
}

impl InstallerError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidLaunch => "installer-invalid-launch",
            Self::DownloadFailed => "installer-download-failed",
            Self::IntegrityFailed => "installer-integrity-failed",
            Self::CleanupFailed => "installer-cleanup-failed",
            Self::InstallFailed => "installer-install-failed",
        }
    }
}
