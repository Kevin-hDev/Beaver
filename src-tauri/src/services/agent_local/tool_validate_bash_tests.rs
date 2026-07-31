use super::validate;
use serde_json::json;

#[test]
fn valid_bash() {
    let args = json!({
        "command": "ls",
        "timeout": 30,
        "yield_time_ms": 500,
        "workdir": "/tmp"
    });
    assert!(validate("bash", &args).is_ok());
}

#[test]
fn valid_bash_write() {
    let args = json!({
        "session_id": "2a8a08dc-660d-477a-9a44-32c24ba814cb",
        "chars": "yes\n",
        "stop": false,
        "yield_time_ms": 500
    });

    assert!(validate("bash_write", &args).is_ok());
    assert!(validate("bash_write", &json!({})).is_err());
    assert!(validate("bash_write", &json!({"session_id": 4})).is_err());
}

#[test]
fn bash_write_accepts_chars_followed_by_eof() {
    let args = json!({
        "session_id": "2a8a08dc-660d-477a-9a44-32c24ba814cb",
        "chars": "payload",
        "eof": true
    });

    let cleaned = validate("bash_write", &args).expect("valid bash_write");

    assert_eq!(cleaned["chars"], "payload");
    assert_eq!(cleaned["eof"], true);
}

#[test]
fn bash_write_rejects_stop_combined_with_input_controls() {
    let session_id = "2a8a08dc-660d-477a-9a44-32c24ba814cb";
    assert!(validate(
        "bash_write",
        &json!({"session_id": session_id, "stop": true, "chars": "x"})
    )
    .is_err());
    assert!(validate(
        "bash_write",
        &json!({"session_id": session_id, "stop": true, "eof": true})
    )
    .is_err());
}

#[test]
fn bash_missing_command() {
    let args = json!({"timeout": 30});
    let err = validate("bash", &args).unwrap_err();
    assert!(err.contains("command"));
}

#[test]
fn bash_rejects_negative_timing_values() {
    assert!(validate("bash", &json!({"command": "pwd", "timeout": -1})).is_err());
    assert!(validate(
        "bash_write",
        &json!({"session_id": "2a8a08dc-660d-477a-9a44-32c24ba814cb", "yield_time_ms": -1}),
    )
    .is_err());
}

#[test]
fn bash_wrong_type() {
    let args = json!({"command": 42});
    let err = validate("bash", &args).unwrap_err();
    assert!(err.contains("string"));
}

#[test]
fn bash_write_rejects_invalid_session_ids_before_dispatch() {
    let error = validate(
        "bash_write",
        &json!({"session_id": "not-a-session", "chars": "hello"}),
    )
    .expect_err("invalid session");

    assert!(error.contains("session_id"));
}

#[test]
fn strips_unknown_args() {
    let args = json!({"command": "ls", "inject": "evil"});
    let cleaned = validate("bash", &args).unwrap();
    assert!(cleaned.get("inject").is_none());
    assert!(cleaned.get("command").is_some());
}

#[test]
fn optional_args_absent() {
    let args = json!({"command": "ls"});
    assert!(validate("bash", &args).is_ok());
}

#[test]
fn bash_workdir_must_be_a_string() {
    let args = json!({"command": "ls", "workdir": 42});
    let err = validate("bash", &args).unwrap_err();
    assert!(err.contains("workdir"));
}
