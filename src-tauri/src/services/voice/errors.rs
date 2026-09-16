use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceErrorCode {
    Busy,
    ShuttingDown,
    InvalidSettings,
    ConfigurationUnavailable,
    InvalidTransition,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VoiceError {
    code: VoiceErrorCode,
}

impl VoiceError {
    pub fn code(&self) -> VoiceErrorCode {
        self.code
    }

    pub fn busy() -> Self {
        Self::from_code(VoiceErrorCode::Busy)
    }

    pub fn shutting_down() -> Self {
        Self::from_code(VoiceErrorCode::ShuttingDown)
    }

    pub fn invalid_settings() -> Self {
        Self::from_code(VoiceErrorCode::InvalidSettings)
    }

    pub fn invalid_transition() -> Self {
        Self::from_code(VoiceErrorCode::InvalidTransition)
    }

    pub fn configuration_unavailable() -> Self {
        Self::from_code(VoiceErrorCode::ConfigurationUnavailable)
    }

    pub(super) fn from_code(code: VoiceErrorCode) -> Self {
        Self { code }
    }
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.code)
    }
}

impl std::error::Error for VoiceError {}
