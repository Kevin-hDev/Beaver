use std::path::Path;

use serde::{Deserialize, Serialize};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, VoiceActivityDetector};
use zeroize::Zeroize;

use crate::services::voice::{
    download::{InstallationReceipt, VoiceCatalogEntry, VoiceEngine},
    errors::VoiceError,
    limits::{MAX_LANGUAGE_CODE_CHARS, MAX_TRANSCRIPT_CHARS},
};

const MAX_SLICE_SAMPLES: usize = 32 * 16_000;
const ENGINE_VERSION: &str = "sherpa-onnx-1.13.8";

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionProfile {
    pub engine_version: String,
    pub backend: String,
    pub num_threads: i32,
}

impl ExecutionProfile {
    pub fn cpu(num_threads: i32) -> Result<Self, VoiceError> {
        if !(1..=64).contains(&num_threads) {
            return Err(VoiceError::invalid_settings());
        }
        Ok(Self {
            engine_version: ENGINE_VERSION.into(),
            backend: "cpu".into(),
            num_threads,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectiveLanguage {
    Automatic,
    Language(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecognitionSlice {
    pub text: String,
    pub tokens: Vec<String>,
    pub timestamps_ms: Option<Vec<u64>>,
    pub durations_ms: Option<Vec<u64>>,
}

impl Drop for RecognitionSlice {
    fn drop(&mut self) {
        self.text.zeroize();
        self.tokens.iter_mut().for_each(Zeroize::zeroize);
    }
}

pub struct PreparedModel {
    pub entry_id: String,
    pub revision: String,
    pub profile: ExecutionProfile,
    engine: VoiceEngine,
    recognizer: OfflineRecognizer,
    vad: VoiceActivityDetector,
}

impl PreparedModel {
    pub(super) fn load(
        entry: &VoiceCatalogEntry,
        receipt: &InstallationReceipt,
        vad_receipt: &InstallationReceipt,
        data_dir: &Path,
        profile: ExecutionProfile,
    ) -> Result<Self, VoiceError> {
        profile.validate()?;
        let root = receipt.install_dir(data_dir);
        let config = match entry.engine {
            VoiceEngine::NemoTransducer => super::parakeet::config(&root, &profile),
            VoiceEngine::CohereTranscribe => super::cohere::config(&root, &profile),
            VoiceEngine::Qwen3Asr => super::qwen::config(&root, &profile),
            VoiceEngine::SileroVad => return Err(VoiceError::configuration_unavailable()),
        };
        let recognizer =
            OfflineRecognizer::create(&config).ok_or_else(VoiceError::configuration_unavailable)?;
        let vad = super::vad::load(&vad_receipt.install_dir(data_dir), &profile)
            .ok_or_else(VoiceError::configuration_unavailable)?;
        Ok(Self {
            entry_id: entry.id.clone(),
            revision: entry.revision.clone(),
            profile,
            engine: entry.engine,
            recognizer,
            vad,
        })
    }

    pub fn vad(&self) -> &VoiceActivityDetector {
        &self.vad
    }
}

pub fn recognize_slice(
    prepared: &mut PreparedModel,
    pcm: &[f32],
    language: &EffectiveLanguage,
) -> Result<RecognitionSlice, VoiceError> {
    if pcm.is_empty()
        || pcm.len() > MAX_SLICE_SAMPLES
        || pcm.iter().any(|sample| !sample.is_finite())
    {
        return Err(VoiceError::invalid_settings());
    }
    let stream = prepared.recognizer.create_stream();
    if prepared.engine == VoiceEngine::CohereTranscribe {
        if let EffectiveLanguage::Language(code) = language {
            validate_language(code)?;
            stream.set_option("language", code);
        }
    }
    stream.accept_waveform(16_000, pcm);
    prepared.recognizer.decode(&stream);
    let result = stream
        .get_result()
        .ok_or_else(VoiceError::configuration_unavailable)?;
    if result.text.chars().count() > MAX_TRANSCRIPT_CHARS {
        return Err(VoiceError::configuration_unavailable());
    }
    if result.tokens.len() > MAX_TRANSCRIPT_CHARS
        || result
            .tokens
            .iter()
            .try_fold(0_usize, |total, token| {
                total.checked_add(token.chars().count())
            })
            .is_none_or(|total| total > MAX_TRANSCRIPT_CHARS)
    {
        return Err(VoiceError::configuration_unavailable());
    }
    Ok(RecognitionSlice {
        text: result.text,
        timestamps_ms: measured_times(result.timestamps, result.tokens.len()),
        durations_ms: measured_times(result.durations, result.tokens.len()),
        tokens: result.tokens,
    })
}

pub(super) fn base_config(profile: &ExecutionProfile) -> OfflineRecognizerConfig {
    let mut config = OfflineRecognizerConfig::default();
    config.model_config.provider = Some(profile.backend.clone());
    config.model_config.num_threads = profile.num_threads;
    config
}

pub(super) fn model_path(root: &Path, relative: &str) -> String {
    root.join(relative).to_string_lossy().into_owned()
}

pub(super) fn measured_times(values: Option<Vec<f32>>, expected: usize) -> Option<Vec<u64>> {
    values
        .filter(|values| {
            values.len() == expected
                && values
                    .iter()
                    .all(|value| value.is_finite() && *value >= 0.0)
        })
        .map(|values| {
            values
                .into_iter()
                .map(|seconds| (seconds * 1_000.0).round() as u64)
                .collect()
        })
}

fn validate_language(code: &str) -> Result<(), VoiceError> {
    if code.is_empty()
        || code.chars().count() > MAX_LANGUAGE_CODE_CHARS
        || !code
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        Err(VoiceError::invalid_settings())
    } else {
        Ok(())
    }
}

impl ExecutionProfile {
    pub(super) fn validate(&self) -> Result<(), VoiceError> {
        if self.engine_version.is_empty()
            || self.engine_version.len() > 32
            || !self
                .engine_version
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
            || self.backend != "cpu"
            || !(1..=64).contains(&self.num_threads)
        {
            Err(VoiceError::invalid_settings())
        } else {
            Ok(())
        }
    }
}
