use std::path::Path;

use sherpa_onnx::{SileroVadModelConfig, VadModelConfig, VoiceActivityDetector};

use super::recognizer::{model_path, ExecutionProfile};

pub(super) fn load(root: &Path, profile: &ExecutionProfile) -> Option<VoiceActivityDetector> {
    let silero_vad = SileroVadModelConfig {
        model: Some(model_path(root, "silero_vad.onnx")),
        threshold: 0.5,
        min_silence_duration: 0.25,
        min_speech_duration: 0.25,
        max_speech_duration: 30.0,
        ..Default::default()
    };
    VoiceActivityDetector::create(
        &VadModelConfig {
            silero_vad,
            sample_rate: 16_000,
            num_threads: 1,
            provider: Some(profile.backend.clone()),
            ..Default::default()
        },
        32.0,
    )
}
