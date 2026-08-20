#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeError {
    ManifestInvalid,
    PythonUnavailable,
    WheelhouseUnavailable,
    EnvironmentUnavailable,
    Cancelled,
}

impl RuntimeError {
    pub(super) fn category(self) -> &'static str {
        match self {
            Self::ManifestInvalid => "manifest-invalid",
            Self::PythonUnavailable => "python-unavailable",
            Self::WheelhouseUnavailable => "wheelhouse-unavailable",
            Self::EnvironmentUnavailable => "environment-unavailable",
            Self::Cancelled => "cancelled",
        }
    }

    pub(super) fn public_message(self) -> &'static str {
        match self {
            Self::Cancelled => "SearXNG: arrêt en cours",
            _ => "SearXNG: runtime indisponible",
        }
    }
}
