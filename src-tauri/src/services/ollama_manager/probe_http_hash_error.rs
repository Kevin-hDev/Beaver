use super::HashFileError;

impl HashFileError {
    pub(crate) const fn diagnostic(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Permission => "permission",
            Self::InvalidFormat => "invalid-format",
            Self::Other => "other",
        }
    }
}
