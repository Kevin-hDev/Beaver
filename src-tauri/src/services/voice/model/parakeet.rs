use std::path::Path;

use sherpa_onnx::{OfflineRecognizerConfig, OfflineTransducerModelConfig};

use super::recognizer::{model_path, ExecutionProfile};

pub(super) fn config(root: &Path, profile: &ExecutionProfile) -> OfflineRecognizerConfig {
    let mut config = super::recognizer::base_config(profile);
    config.model_config.transducer = OfflineTransducerModelConfig {
        encoder: Some(model_path(root, "encoder.int8.onnx")),
        decoder: Some(model_path(root, "decoder.int8.onnx")),
        joiner: Some(model_path(root, "joiner.int8.onnx")),
    };
    config.model_config.tokens = Some(model_path(root, "tokens.txt"));
    config.model_config.model_type = Some("nemo_transducer".into());
    config
}
