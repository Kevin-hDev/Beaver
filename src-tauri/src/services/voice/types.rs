use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoicePhase {
    Idle,
    Preparing,
    Listening,
    Transcribing,
    Recovering,
    Stopping,
    Delivering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceModel {
    ParakeetTdtV3,
    CohereTranscribe,
    Qwen3Asr06b,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum VoiceInputDevice {
    SystemDefault,
    Device(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceInputGain {
    #[serde(rename = "zero-db")]
    Zero,
    #[serde(rename = "three-db")]
    Three,
    #[default]
    #[serde(rename = "six-db")]
    Six,
    #[serde(rename = "nine-db")]
    Nine,
}

impl VoiceInputGain {
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Zero => 1.0,
            Self::Three => 1.412_537_6,
            Self::Six => 1.995_262_3,
            Self::Nine => 2.818_383,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceSilenceTimeout {
    ThreeSeconds,
    FiveSeconds,
    TenSeconds,
    TwentySeconds,
    ThirtySeconds,
    Never,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum VoiceMaxDuration {
    #[serde(rename = "2-minutes")]
    Two,
    #[serde(rename = "5-minutes")]
    Five,
    #[serde(rename = "10-minutes")]
    Ten,
    #[serde(rename = "20-minutes")]
    Twenty,
    #[serde(rename = "30-minutes")]
    Thirty,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum VoiceLanguage {
    FollowInterface,
    Automatic,
    Language(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceUnloadDelay {
    Immediately,
    OneMinute,
    TwoMinutes,
    FiveMinutes,
    FifteenMinutes,
    OnExit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VoiceSettings {
    pub version: u8,
    pub enabled: bool,
    pub model: VoiceModel,
    pub input_device: VoiceInputDevice,
    #[serde(default)]
    pub input_gain: VoiceInputGain,
    pub silence_timeout: VoiceSilenceTimeout,
    pub max_duration: VoiceMaxDuration,
    pub language: VoiceLanguage,
    pub shortcut: Option<String>,
    pub unload_delay: VoiceUnloadDelay,
    pub explanation_accepted: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            version: super::limits::SETTINGS_VERSION,
            enabled: true,
            model: VoiceModel::ParakeetTdtV3,
            input_device: VoiceInputDevice::SystemDefault,
            input_gain: VoiceInputGain::default(),
            silence_timeout: VoiceSilenceTimeout::FiveSeconds,
            max_duration: VoiceMaxDuration::Ten,
            language: VoiceLanguage::Automatic,
            shortcut: None,
            unload_delay: VoiceUnloadDelay::TwoMinutes,
            explanation_accepted: false,
        }
    }
}

impl VoiceSettings {
    pub(crate) fn normalized(mut self) -> Option<Self> {
        if self.version != super::limits::SETTINGS_VERSION {
            return None;
        }
        let patch = VoiceSettingsPatch {
            input_device: Some(self.input_device.clone()),
            input_gain: Some(self.input_gain),
            language: Some(self.language.clone()),
            shortcut: Some(self.shortcut.clone()),
            ..Default::default()
        };
        patch.validate().ok()?;
        self.language = super::language::normalize_for_model(self.model, self.language);
        Some(self)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(optional_fields))]
#[serde(default, deny_unknown_fields)]
pub struct VoiceSettingsPatch {
    pub enabled: Option<bool>,
    pub model: Option<VoiceModel>,
    pub input_device: Option<VoiceInputDevice>,
    pub input_gain: Option<VoiceInputGain>,
    pub silence_timeout: Option<VoiceSilenceTimeout>,
    pub max_duration: Option<VoiceMaxDuration>,
    pub language: Option<VoiceLanguage>,
    #[cfg_attr(test, ts(optional, type = "string | null"))]
    #[serde(default, deserialize_with = "super::nullable::deserialize")]
    pub shortcut: Option<Option<String>>,
    pub unload_delay: Option<VoiceUnloadDelay>,
    pub explanation_accepted: Option<bool>,
}

impl VoiceSettingsPatch {
    pub fn validate(&self) -> Result<(), super::errors::VoiceError> {
        if let Some(VoiceInputDevice::Device(id)) = &self.input_device {
            validate_bounded(id, super::limits::MAX_DEVICE_ID_CHARS)?;
        }
        if let Some(VoiceLanguage::Language(code)) = &self.language {
            validate_bounded(code, super::limits::MAX_LANGUAGE_CODE_CHARS)?;
            if !code
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
            {
                return Err(super::errors::VoiceError::invalid_settings());
            }
        }
        if let Some(Some(shortcut)) = &self.shortcut {
            validate_bounded(shortcut, super::limits::MAX_SHORTCUT_CHARS)?;
        }
        Ok(())
    }

    pub(super) fn apply(self, settings: &mut VoiceSettings) {
        macro_rules! replace {
            ($field:ident) => {
                if let Some(value) = self.$field {
                    settings.$field = value;
                }
            };
        }
        replace!(enabled);
        replace!(model);
        replace!(input_device);
        replace!(input_gain);
        replace!(silence_timeout);
        replace!(max_duration);
        replace!(language);
        replace!(shortcut);
        replace!(unload_delay);
        replace!(explanation_accepted);
        settings.language =
            super::language::normalize_for_model(settings.model, settings.language.clone());
        settings.version = super::limits::SETTINGS_VERSION;
    }
}

fn validate_bounded(value: &str, max_chars: usize) -> Result<(), super::errors::VoiceError> {
    if value.is_empty() || value.chars().count() > max_chars || value.chars().any(char::is_control)
    {
        Err(super::errors::VoiceError::invalid_settings())
    } else {
        Ok(())
    }
}
