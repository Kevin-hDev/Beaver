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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceErrorParamKey {
    RequiredBytes,
    AvailableBytes,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VoiceErrorParam {
    pub key: VoiceErrorParamKey,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VoiceError {
    code: VoiceErrorCode,
    params: Vec<VoiceErrorParam>,
}

impl VoiceError {
    pub fn with_params(code: VoiceErrorCode, params: Vec<VoiceErrorParam>) -> Result<Self, Self> {
        if params.len() > super::limits::MAX_ERROR_PARAMS
            || params.iter().any(|param| {
                param.value.chars().count() > super::limits::MAX_ERROR_PARAM_CHARS
                    || param.value.chars().any(char::is_control)
            })
        {
            return Err(Self::invalid_settings());
        }
        Ok(Self { code, params })
    }

    pub fn code(&self) -> VoiceErrorCode {
        self.code
    }

    pub fn params(&self) -> &[VoiceErrorParam] {
        &self.params
    }

    pub fn busy() -> Self {
        Self::from_code(VoiceErrorCode::Busy)
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
        Self {
            code,
            params: Vec::new(),
        }
    }
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.code)
    }
}

impl std::error::Error for VoiceError {}
