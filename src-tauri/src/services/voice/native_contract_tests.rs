#[test]
fn sherpa_native_contracts_compile_and_resample() {
    use sherpa_onnx::{
        LinearResampler, OfflineCohereTranscribeModelConfig, OfflineQwen3ASRModelConfig,
        OfflineRecognizerResult, OfflineTransducerModelConfig, VoiceActivityDetector,
    };

    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<VoiceActivityDetector>();
    assert_send_sync::<super::capture::stream::CaptureStream>();
    assert!(std::mem::needs_drop::<VoiceActivityDetector>());
    assert!(std::mem::needs_drop::<LinearResampler>());

    let _parakeet = OfflineTransducerModelConfig::default();
    let cohere = OfflineCohereTranscribeModelConfig {
        language: Some("fr".into()),
        ..Default::default()
    };
    let _qwen = OfflineQwen3ASRModelConfig::default();
    assert_eq!(cohere.language.as_deref(), Some("fr"));

    let result = OfflineRecognizerResult {
        text: String::new(),
        tokens: Vec::new(),
        timestamps: Some(vec![0.0]),
        durations: Some(vec![0.1]),
    };
    assert_eq!(result.timestamps.as_deref(), Some([0.0].as_slice()));

    let resampler = LinearResampler::create(48_000, 16_000).expect("native resampler");
    let output = resampler.resample(&vec![0.25; 4_800], true);
    assert!(output.len() >= 1_590 && output.len() <= 1_610);
}
