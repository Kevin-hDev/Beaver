use super::*;

#[test]
fn diagnostic_redacts_credentials_in_retained_error_fields() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join(FILE_NAME);
    let body = serde_json::json!({"error": {
        "type": "sk-test-sentinel-type-12345678",
        "code": "sk-test-sentinel-code-12345678",
        "param": "sk-test-sentinel-param-12345678",
        "metadata": {"provider_name": "sk-test-sentinel-provider-12345678"}
    }})
    .to_string();
    let mut diagnostic = entry(ProviderDiagnosticContext::from_payload(
        None,
        &serde_json::Value::Null,
    ));
    diagnostic.details = super::super::provider_error::safe_details(&body);
    write_at(&path, &diagnostic).unwrap();
    let stored = std::fs::read_to_string(path).unwrap();
    assert!(!stored.contains("sk-test-sentinel"));
}

fn entry(context: ProviderDiagnosticContext) -> ProviderDiagnostic {
    ProviderDiagnostic {
        timestamp: "safe".to_string(),
        provider: safe_identifier("openai\nignored"),
        model: safe_identifier("gpt-5"),
        status: 400,
        details: SafeProviderDetails {
            error_type: Some("invalid_request".to_string()),
            error_code: Some("bad_schema".to_string()),
            error_param: Some("tools[0]".to_string()),
            ..Default::default()
        },
        request_bytes: 100,
        tool_count: 2,
        request_id: context.request_id,
        output_limit: context.output_limit,
    }
}

#[test]
fn diagnostic_is_bounded_and_contains_only_safe_fields() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join(FILE_NAME);
    let entry = entry(ProviderDiagnosticContext::from_payload(
        None,
        &serde_json::json!({}),
    ));
    let mut line = serde_json::to_vec(&entry).unwrap();
    line.push(b'\n');
    let initial = line.repeat(MAX_LOG_BYTES / line.len());
    std::fs::write(&path, initial).unwrap();
    write_at(&path, &entry).unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.len() <= MAX_LOG_BYTES);
    assert!(!text.contains('\n') || text.ends_with('\n'));
    assert!(!text.contains("ignored"));
}

#[test]
fn diagnostic_keeps_valid_request_and_serialized_output_limit_only() {
    let context = ProviderDiagnosticContext::from_payload(
        Some("request-123"),
        &serde_json::json!({"max_completion_tokens": 321, "max_tokens": "private"}),
    );
    let value = serde_json::to_value(entry(context)).unwrap();

    assert_eq!(value["request_id"], "request-123");
    assert_eq!(value["output_limit"]["field"], "max_completion_tokens");
    assert_eq!(value["output_limit"]["value"], 321);
}

#[test]
fn malformed_request_and_missing_output_limit_stay_unknown() {
    let context = ProviderDiagnosticContext::from_payload(
        Some("Bearer secret\nrequest"),
        &serde_json::json!({"max_tokens": 0, "private_limit": 999}),
    );
    let text = serde_json::to_string(&entry(context)).unwrap();

    assert!(!text.contains("request_id"));
    assert!(!text.contains("output_limit"));
    assert!(!text.contains("secret"));
    assert!(!text.contains("999"));
}

#[test]
fn diagnostic_round_trip_is_utf8_safe_and_never_persists_a_secret_body() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join(FILE_NAME);
    let context = ProviderDiagnosticContext::from_serialized(
        Some("request-utf8"),
        r#"{"max_output_tokens":512,"prompt":"秘密 secret-sentinel"}"#,
    );
    let mut diagnostic = entry(context);
    diagnostic.model = safe_identifier("modèle-épreuve");

    write_at(&path, &diagnostic).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(text.trim()).unwrap();

    assert_eq!(value["request_id"], "request-utf8");
    assert_eq!(value["output_limit"]["field"], "max_output_tokens");
    assert_eq!(value["output_limit"]["value"], 512);
    assert_eq!(value["model"], "unknown");
    assert!(!text.contains("秘密"));
    assert!(!text.contains("secret-sentinel"));
}

#[test]
fn diagnostic_keeps_the_same_valid_routed_model_id_as_runtime() {
    let mut diagnostic = entry(ProviderDiagnosticContext::from_payload(
        None,
        &serde_json::Value::Null,
    ));
    diagnostic.model = safe_model_identifier("google/gemma-4-31b-it:free");

    assert_eq!(diagnostic.model, "google/gemma-4-31b-it:free");
}
