use std::path::Path;

use sherpa_onnx::{OfflineCohereTranscribeModelConfig, OfflineRecognizerConfig};

use super::recognizer::{model_path, ExecutionProfile};

pub(super) fn config(root: &Path, profile: &ExecutionProfile) -> OfflineRecognizerConfig {
    let mut config = super::recognizer::base_config(profile);
    config.model_config.cohere_transcribe = OfflineCohereTranscribeModelConfig {
        encoder: Some(model_path(root, "encoder.int8.onnx")),
        decoder: Some(model_path(root, "decoder.int8.onnx")),
        use_punct: true,
        use_itn: true,
        ..Default::default()
    };
    config.model_config.tokens = Some(model_path(root, "tokens.txt"));
    config
}
