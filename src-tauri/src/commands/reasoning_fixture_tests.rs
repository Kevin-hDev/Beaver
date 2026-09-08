use super::*;

#[test]
fn report_never_serializes_a_session_id() {
    let report = SanitizedFixtureReport {
        schema_version: 1,
        fixture_id: "ollama-local-test-us-east-1-2026-08-26".to_string(),
        route: "ollama".to_string(),
        model: "test".to_string(),
        region: "us-east-1".to_string(),
        reasoning_mode: "auto".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        scenarios: Vec::new(),
    };
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("session_id"));
}

#[test]
fn new_models_use_mode_ids_while_historic_ids_keep_their_shape() {
    assert_eq!(
        super::fixture_report_variant("google", "gemini-3.8-flash", "low"),
        Some("low")
    );
    assert_eq!(
        super::fixture_report_variant("google", "gemini-3.8-flash", "high"),
        Some("high")
    );
    assert_ne!(
        crate::services::reasoning_fixture_store::derive_fixture_id_with_variant(
            "google",
            "gemini-3.8-flash",
            "low",
            "france",
            chrono::NaiveDate::from_ymd_opt(2026, 9, 7).unwrap(),
        )
        .unwrap(),
        crate::services::reasoning_fixture_store::derive_fixture_id_with_variant(
            "google",
            "gemini-3.8-flash",
            "high",
            "france",
            chrono::NaiveDate::from_ymd_opt(2026, 9, 7).unwrap(),
        )
        .unwrap()
    );
    assert_eq!(
        super::fixture_report_variant("google", "gemini-3.5-flash", "medium"),
        None
    );
    assert_eq!(
        crate::services::reasoning_fixture_store::derive_fixture_id(
            "google",
            "gemini-3.5-flash",
            "france",
            chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap(),
        )
        .unwrap(),
        "google-api-gemini-3-5-flash-france-2026-08-26"
    );
}

#[tokio::test]
async fn bounded_fixture_operations_run_only_through_the_isolated_toolset() {
    let results = run_fixture_operations(vec![
        FixtureOperation {
            tool_id: "fixture.write_note".to_string(),
            arguments: serde_json::json!({ "value": "fixture" }),
        },
        FixtureOperation {
            tool_id: "fixture.read_note".to_string(),
            arguments: serde_json::json!({}),
        },
    ])
    .await
    .expect("fixture operations");

    assert_eq!(
        results,
        vec![
            serde_json::json!({ "written": true }),
            serde_json::json!({ "value": "fixture" })
        ]
    );
}
