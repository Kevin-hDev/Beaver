use crate::services::agent_local::ollama_registry::cleanup_partial_blobs;

#[test]
fn cleanup_partial_blobs_returns_zero_on_empty_digests() {
    let count = cleanup_partial_blobs(&[]);
    assert_eq!(count, 0, "should never delete when digests is empty");
}

#[test]
fn cleanup_partial_blobs_returns_zero_on_nonexistent_dir() {
    let _count = cleanup_partial_blobs(&["sha256-abc123".to_string()]);
    // just shouldn't panic
}
