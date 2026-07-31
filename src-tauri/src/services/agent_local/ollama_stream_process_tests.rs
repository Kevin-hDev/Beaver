use super::ollama_stream_process::done_generation_duration;

#[test]
fn reads_bounded_native_ollama_generation_duration() {
    let chunk = serde_json::json!({ "eval_duration": 2_500_000_000_u64 });

    assert_eq!(done_generation_duration(&chunk), Some(2_500_000_000));
}

#[test]
fn rejects_invalid_native_ollama_generation_duration() {
    assert_eq!(done_generation_duration(&serde_json::json!({ "eval_duration": 0 })), None);
}
