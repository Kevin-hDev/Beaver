use super::*;

fn line(id: &str, fired_at: &str, status: WakeupRunStatus) -> String {
    serde_json::to_string(&WakeupRun {
        wakeup_id: id.into(),
        scheduled_for: "2026-05-17T08:00:00+02:00".into(),
        fired_at: fired_at.into(),
        status,
        error: None,
        session_id: None,
        tokens: None,
    })
    .unwrap()
}

#[test]
fn parse_runs_filters_and_sorts_newest_first() {
    let content = format!(
        "{}\n{}\n",
        line("a", "2026-05-17T08:00:00Z", WakeupRunStatus::Ok),
        line("b", "2026-05-17T09:00:00Z", WakeupRunStatus::Missed)
    );
    let runs = parse_runs(&content, None);
    assert_eq!(runs[0].wakeup_id, "b");
    assert_eq!(parse_runs(&content, Some("a")).len(), 1);
}

#[test]
fn generic_error_does_not_return_raw_message() {
    assert_eq!(generic_error("token secret leaked"), "Le réveil a échoué");
    assert_eq!(generic_error("Ollama HTTP 500"), "Ollama indisponible");
}

#[test]
fn generic_error_maps_known_failures() {
    assert_eq!(
        generic_error("RATE LIMIT hit"),
        "Limite de requêtes atteinte"
    );
    assert_eq!(
        generic_error("401 unauthorized"),
        "Authentification échouée"
    );
    assert_eq!(
        generic_error("invalid api key sk-xxx"),
        "Le réveil a échoué"
    );
}

#[test]
fn generic_error_never_returns_sensitive_input() {
    let sensitive_inputs = [
        "Bearer sk-abcd1234efgh",
        "Connection refused to 127.0.0.1:11434",
        "panic at src/main.rs:42",
        "/Users/secret/.config/keys.json not found",
    ];
    for input in sensitive_inputs {
        let result = generic_error(input);
        assert_ne!(result, input);
        assert!(!result.contains("sk-"));
        assert!(!result.contains("127.0.0.1"));
        assert!(!result.contains("panic"));
        assert!(!result.contains("/Users/"));
    }
}

#[test]
fn parse_runs_is_bounded_before_collection() {
    let mut content = String::new();
    for i in 0..600 {
        let fired_at = format!("2026-05-17T{i:04}:00:00Z");
        content.push_str(&line(&format!("w{i}"), &fired_at, WakeupRunStatus::Ok));
        content.push('\n');
    }
    let runs = parse_runs(&content, None);
    assert_eq!(runs.len(), MAX_LINES);
    assert!(runs.iter().all(|run| run.wakeup_id != "w0"));
}

#[test]
fn ids_are_ascii_bounded_and_control_characters_are_removed() {
    let id = format!("{}\nsecret", "a".repeat(MAX_ID_CHARS + 20));
    let safe = safe_id(&id);
    assert_eq!(safe.len(), MAX_ID_CHARS);
    assert!(!safe.contains('\n'));
}

#[test]
fn oversized_lines_are_never_deserialized() {
    let oversized = format!("{{\"wakeup_id\":\"{}\"}}", "a".repeat(MAX_LOG_LINE_BYTES));
    assert!(parse_runs(&oversized, None).is_empty());
}
