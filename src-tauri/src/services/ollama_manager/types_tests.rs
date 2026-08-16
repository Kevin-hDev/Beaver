use super::super::error::OllamaErrorCode;
use super::super::types::{
    BundleState, DaemonState, OllamaEndpoint, OllamaProgressStage, OllamaRuntimeStatus,
    OllamaStartOutcome, OperationState,
};
use serde_json::json;
use std::num::NonZeroU16;

#[test]
fn progress_is_a_closed_typed_set_of_stages() {
    let stages = [
        OllamaProgressStage::Preparing,
        OllamaProgressStage::Downloading,
        OllamaProgressStage::Verifying,
        OllamaProgressStage::Extracting,
        OllamaProgressStage::Validating,
        OllamaProgressStage::Committing,
        OllamaProgressStage::Starting,
        OllamaProgressStage::Recovering,
        OllamaProgressStage::RollingBack,
        OllamaProgressStage::Cleaning,
    ];
    assert_eq!(stages.len(), 10);
    assert_eq!(
        serde_json::to_value(stages[0]).expect("serialize stage"),
        json!("preparing")
    );
}

#[test]
fn endpoint_is_strict_loopback_http() {
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11434).expect("non-zero port"));
    assert_eq!(endpoint.as_http_url(), "http://127.0.0.1:11434");

    for invalid in [
        "http://127.0.0.1:0",
        "http://user:pass@127.0.0.1:11434",
        "http://127.0.0.1:11434?token=secret",
        "http://127.0.0.1:11434/#fragment",
        "http://127.0.0.2:11434",
        "http://example.test:11434",
    ] {
        assert!(
            OllamaEndpoint::try_from_http_url(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn ipc_types_serialize_and_generate_typescript() {
    let status = OllamaRuntimeStatus {
        bundle: BundleState::Absent,
        daemon: DaemonState::Unavailable,
        operation: OperationState::Idle,
        progress: None,
        last_error: None,
    };
    let outcome = OllamaStartOutcome::RejectedDuringShutdown;
    assert_eq!(
        serde_json::to_value(status).expect("serialize status"),
        json!({
            "bundle": "absent",
            "daemon": "unavailable",
            "operation": "idle",
            "progress": null,
            "last_error": null,
        })
    );
    assert_eq!(
        serde_json::to_value(outcome).expect("serialize outcome"),
        json!("rejected_during_shutdown")
    );

    use ts_rs::{Config, TS};
    let config = Config::default();
    assert!(!BundleState::decl(&config).is_empty());
    assert!(!DaemonState::decl(&config).is_empty());
    assert!(!OperationState::decl(&config).is_empty());
    assert!(!OllamaProgressStage::decl(&config).is_empty());
    assert!(!OllamaEndpoint::decl(&config).is_empty());
    assert!(!OllamaRuntimeStatus::decl(&config).is_empty());
    assert!(!OllamaStartOutcome::decl(&config).is_empty());
    assert!(!OllamaErrorCode::decl(&config).is_empty());
}

#[test]
fn public_error_codes_are_exactly_twenty_and_kebab_case() {
    let codes = OllamaErrorCode::ALL;
    assert_eq!(codes.len(), 20);
    let serialized = codes
        .into_iter()
        .map(|code| serde_json::to_string(&code).expect("serialize code"))
        .collect::<Vec<_>>();
    assert!(serialized.iter().all(|code| code.starts_with('"')));
    assert!(serialized.iter().all(|code| !code.contains("Ollama")));
    assert!(serialized.contains(&"\"ollama-operation-in-progress\"".to_string()));
}
