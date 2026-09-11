use serde_json::Value;

const HISTORICAL_DIAGNOSTIC: &[u8] = include_bytes!(
    "../../../test-fixtures/agent-session-v5-circuit-breaker-unknown.json"
);

#[tokio::test]
async fn historical_circuit_breaker_diagnostic_survives_reload() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000071.json");
    crate::services::private_store::atomic_write(&path, HISTORICAL_DIAGNOSTIC)
        .expect("write historical session");

    let session = super::session_store_document::read_from_path(path)
        .await
        .expect("reload historical session");
    let view = super::session_view::from_session(&session).expect("project session view");
    let run = view.diagnostic_runs.first().expect("diagnostic run");

    assert_eq!(run["error_type"], Value::String("circuit_breaker".into()));
    assert_eq!(
        run["safe_summary"],
        Value::String(
            "Interruption après le dernier tool bash_control (circuit_breaker).".into()
        )
    );
    assert_eq!(
        run["events"][0]["error_type"],
        Value::String("circuit_breaker".into())
    );
}
