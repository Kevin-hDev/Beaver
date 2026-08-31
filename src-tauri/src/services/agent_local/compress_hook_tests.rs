#[test]
fn ollama_compression_reads_the_profile_store_not_legacy_config() {
    let source = include_str!("compress_hook.rs");
    assert!(source.contains("orchestrator::run_compression"));
    assert!(!source.contains("config::read_config"));
}
