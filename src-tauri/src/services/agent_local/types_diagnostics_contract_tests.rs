use super::tool_result_contract::typescript_bindings as tool_result_typescript_bindings;
use super::types_diagnostics::typescript_bindings;

#[test]
fn checked_in_typescript_matches_the_rust_diagnostics_contract() {
    let checked_in = include_str!("../../../../src/types/agent-diagnostics.ts").replace("\r\n", "\n");

    assert_eq!(checked_in, typescript_bindings());
    assert!(checked_in.contains("from \"./agent-tool-result-contract\""));

    let tool_result_checked_in =
        include_str!("../../../../src/types/agent-tool-result-contract.ts").replace("\r\n", "\n");
    assert_eq!(tool_result_checked_in, tool_result_typescript_bindings());
}

#[test]
fn legacy_diagnostic_run_without_attempt_identity_remains_readable() {
    let run: super::types_diagnostics::AgentDiagnosticRun = serde_json::from_value(
        serde_json::json!({
            "request_id": "legacy-request",
            "generation": 1,
            "status": "failed",
            "severity": "error",
            "started_at": "2026-09-09T00:00:00Z",
            "updated_at": "2026-09-09T00:00:01Z",
            "phase": "failed"
        }),
    )
    .unwrap();

    assert!(run.provider.is_none());
    assert!(run.model.is_none());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_diagnostics_contract() {
    let types_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types");
    let diagnostics_path = types_dir.join("agent-diagnostics.ts");
    let tool_result_path = types_dir.join("agent-tool-result-contract.ts");

    std::fs::write(diagnostics_path, typescript_bindings()).unwrap();
    std::fs::write(tool_result_path, tool_result_typescript_bindings()).unwrap();
}
