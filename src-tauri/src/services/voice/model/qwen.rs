use std::path::Path;

use sherpa_onnx::{OfflineQwen3ASRModelConfig, OfflineRecognizerConfig};

use super::recognizer::{model_path, ExecutionProfile};

pub(super) fn config(root: &Path, profile: &ExecutionProfile) -> OfflineRecognizerConfig {
    let mut config = super::recognizer::base_config(profile);
    config.model_config.qwen3_asr = OfflineQwen3ASRModelConfig {
        conv_frontend: Some(model_path(root, "conv_frontend.onnx")),
        encoder: Some(model_path(root, "encoder.int8.onnx")),
        decoder: Some(model_path(root, "decoder.int8.onnx")),
        tokenizer: Some(model_path(root, "tokenizer")),
        ..Default::default()
    };
    config.model_config.tokens = Some(String::new());
    config
}
