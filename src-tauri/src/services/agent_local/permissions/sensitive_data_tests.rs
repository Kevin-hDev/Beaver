use super::*;
use serde_json::json;

#[test]
fn sensitive_path_is_not_eligible_for_historical_content() {
    assert!(is_sensitive_path(std::path::Path::new("/repo/.env")));
    assert!(is_sensitive_path(std::path::Path::new(
        "/home/user/.ssh/id_ed25519"
    )));
    assert!(!is_sensitive_path(std::path::Path::new(
        "/repo/src/main.rs"
    )));
}

#[test]
fn detects_sensitive_paths_but_ignores_heredoc_bodies() {
    assert!(bash_touches_sensitive_data("cat .env"));
    assert!(bash_touches_sensitive_data("head ~/.ssh/id_ed25519"));
    assert!(bash_touches_sensitive_data(
        "cat ~/.local/share/cl-go-dash/secrets.enc"
    ));
    assert!(!bash_touches_sensitive_data("grep -r token src/"));
    let heredoc = "cat > README.md << 'EOF'\nSee ~/.ssh/id_rsa and .env\nEOF";
    assert!(!bash_touches_sensitive_data(heredoc));
}

#[test]
fn redacts_assignments_and_json() {
    let text = "API_KEY=abcd PASSWORD: hunter2";
    let redacted = redact_text(text);
    assert!(!redacted.contains("abcd"));
    assert!(!redacted.contains("hunter2"));
    let value = json!({ "command": "echo token=abcdefghi" });
    assert!(!redact_json(&value).to_string().contains("abcdefghi"));
}

#[test]
fn redacts_all_supported_credential_shapes() {
    let fixtures = [
        ["123456789", ":", &"A".repeat(35)].concat(),
        ["xapp", "-1-", &"B".repeat(30)].concat(),
        ["xoxb", "-", &"C".repeat(30)].concat(),
        ["sk", "-proj-", &"D".repeat(24)].concat(),
        ["github", "_pat_", &"E".repeat(24)].concat(),
        ["AK", "IA", &"F".repeat(16)].concat(),
        ["AI", "za", &"K".repeat(35)].concat(),
        ["gsk", "_", &"L".repeat(24)].concat(),
        ["xai", "-", &"M".repeat(24)].concat(),
        ["csk", "-", &"N".repeat(24)].concat(),
        ["hf", "_", &"P".repeat(24)].concat(),
        [
            "https://hooks.slack.com/services/",
            "T00000000/B00000000/",
            &"S".repeat(24),
        ]
        .concat(),
        format!("{}.{}.{}", "G".repeat(24), "H".repeat(6), "I".repeat(28)),
        ["Bear", "er ", &"J".repeat(24)].concat(),
    ];
    for (index, fixture) in fixtures.into_iter().enumerate() {
        let redacted = redact_text(&format!("échec: {fixture}"));
        assert!(!redacted.contains(&fixture), "fixture {index}");
        assert!(redacted.contains("[REDACTED]"), "fixture {index}");
    }
}

#[test]
fn keeps_common_words_and_urls_intact() {
    let text = "Desk-ask-risk-task https://example.test/task-report";
    assert_eq!(redact_text(text), text);
}

#[test]
fn preserving_redaction_keeps_json_shape() {
    let mut value = json!({
        "messages": [
            {"content": ["hello", "gsk_1234567890abcdefghijkl"]},
            {"nested": {"token": "private-value"}}
        ]
    });
    redact_json_preserving_shape(&mut value);
    assert_eq!(value["messages"].as_array().unwrap().len(), 2);
    assert_eq!(value["messages"][0]["content"].as_array().unwrap().len(), 2);
    assert_eq!(value["messages"][0]["content"][0], "hello");
    assert_eq!(value["messages"][0]["content"][1], "[REDACTED]");
    assert_eq!(value["messages"][1]["nested"]["token"], "[REDACTED]");
}
