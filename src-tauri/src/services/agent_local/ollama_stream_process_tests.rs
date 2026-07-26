use super::ollama_stream_process::done_counts;

#[test]
fn done_counts_keeps_missing_counts_as_none() {
    let chunk = serde_json::json!({ "done": true });

    let counts = done_counts(&chunk);

    assert_eq!(counts.eval_count, None);
    assert_eq!(counts.prompt_tokens, None);
    assert_eq!(counts.context_tokens, None);
}

#[test]
fn done_counts_keeps_partial_counts_as_none_for_context() {
    let chunk = serde_json::json!({
        "done": true,
        "eval_count": 7
    });

    let counts = done_counts(&chunk);

    assert_eq!(counts.eval_count, Some(7));
    assert_eq!(counts.prompt_tokens, None);
    assert_eq!(counts.context_tokens, None);
}

#[test]
fn done_counts_uses_native_ollama_counts_when_present() {
    let chunk = serde_json::json!({
        "done": true,
        "eval_count": 7,
        "prompt_eval_count": 11
    });

    let counts = done_counts(&chunk);

    assert_eq!(counts.eval_count, Some(7));
    assert_eq!(counts.prompt_tokens, Some(11));
    assert_eq!(counts.context_tokens, Some(18));
}
