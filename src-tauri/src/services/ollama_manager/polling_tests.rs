use super::polling::{build_gpu_status, PsResponse};

#[test]
fn gpu_with_system_vram() {
    let json = r#"{"models":[{"name":"qwen3:14b","size":9000000000,"size_vram":9000000000}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    let status = build_gpu_status(&ps, 24000, 19000);
    assert_eq!(status.accelerator, "GPU");
    assert_eq!(status.vram_used_mb, 19000);
    assert_eq!(status.vram_total_mb, 24000);
}

#[test]
fn cpu_fallback() {
    let json = r#"{"models":[{"name":"gemma4:e4b","size":3000000000,"size_vram":0}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(build_gpu_status(&ps, 0, 0).accelerator, "CPU");
}

#[test]
fn gpu_with_system_usage_without_total() {
    let json = r#"{"models":[{"name":"qwen3:14b","size":9000000000,"size_vram":5368709120}]}"#;
    let ps: PsResponse = serde_json::from_str(json).unwrap();
    let status = build_gpu_status(&ps, 0, 5120);
    assert_eq!(status.accelerator, "GPU");
    assert_eq!(status.vram_used_mb, 5120);
}

#[test]
fn idle_with_vram_shows_gpu() {
    let status = build_gpu_status(&PsResponse { models: vec![] }, 16000, 0);
    assert_eq!(status.accelerator, "GPU");
    assert!(status.model_loaded.is_none());
}

#[test]
fn idle_without_vram_shows_cpu() {
    let status = build_gpu_status(&PsResponse { models: vec![] }, 0, 0);
    assert_eq!(status.accelerator, "CPU");
    assert!(status.model_loaded.is_none());
}

#[test]
fn polling_only_observes_and_never_restarts() {
    let source = include_str!("polling.rs");
    assert!(!source.contains("start_sidecar"));
    assert!(!source.contains("restart"));
    assert!(!source.contains("update_ollama"));
}
