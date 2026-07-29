use super::validate;
use serde_json::json;

#[test]
fn valid_bash() {
    let args = json!({"command": "ls", "timeout": 30, "workdir": "/tmp"});
    assert!(validate("bash", &args).is_ok());
}

#[test]
fn bash_missing_command() {
    let args = json!({"timeout": 30});
    let err = validate("bash", &args).unwrap_err();
    assert!(err.contains("command"));
}

#[test]
fn bash_wrong_type() {
    let args = json!({"command": 42});
    let err = validate("bash", &args).unwrap_err();
    assert!(err.contains("string"));
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
