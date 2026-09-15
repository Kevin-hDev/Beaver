use super::{
    cohere, parakeet, qwen,
    recognizer::{append_inference_silence, measured_times, ExecutionProfile},
};

#[test]
fn execution_profile_rejects_unbounded_native_threads() {
    assert!(ExecutionProfile::cpu(0).is_err());
    assert!(ExecutionProfile::cpu(2).is_ok());
    assert!(ExecutionProfile::cpu(65).is_err());
}

#[test]
fn each_engine_adapter_uses_only_its_manifest_paths() {
    let root = std::path::Path::new("/verified/model");
    let profile = ExecutionProfile::cpu(2).unwrap();

    let parakeet = parakeet::config(root, &profile);
    assert_eq!(
        parakeet.model_config.transducer.encoder.as_deref(),
        Some("/verified/model/encoder.int8.onnx")
    );
    assert_eq!(
        parakeet.model_config.model_type.as_deref(),
        Some("nemo_transducer")
    );

    let cohere = cohere::config(root, &profile);
    assert!(cohere.model_config.cohere_transcribe.use_punct);
    assert!(cohere.model_config.cohere_transcribe.use_itn);
    assert!(cohere.model_config.cohere_transcribe.language.is_none());

    let qwen = qwen::config(root, &profile);
    assert_eq!(
        qwen.model_config.qwen3_asr.tokenizer.as_deref(),
        Some("/verified/model/tokenizer")
    );
    assert_eq!(qwen.model_config.tokens.as_deref(), Some(""));
}

#[test]
fn timestamps_are_kept_only_when_the_native_shape_is_usable() {
    assert_eq!(
        measured_times(Some(vec![0.0, 1.25]), 2),
        Some(vec![0, 1_250])
    );
    assert_eq!(measured_times(Some(vec![0.0]), 2), None);
    assert_eq!(measured_times(Some(vec![f32::NAN]), 1), None);
}

#[test]
fn inference_gets_a_quarter_second_to_finalize_its_last_word() {
    let padded = append_inference_silence(&[0.25, -0.25]).unwrap();
    assert_eq!(&padded[..2], &[0.25, -0.25]);
    assert_eq!(padded.len(), 4_002);
    assert!(padded[2..].iter().all(|sample| *sample == 0.0));
}
