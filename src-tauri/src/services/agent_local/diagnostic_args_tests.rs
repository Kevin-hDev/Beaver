use super::diagnostic_args::summarize;
use serde_json::json;
use std::path::Path;

#[test]
fn file_args_keep_relative_path_only() {
    let out = summarize(
        "write_file",
        &json!({"path": "src/main.rs", "content": "secret"}),
        Path::new("/tmp/project"),
    )
    .unwrap();
    assert_eq!(out["path"], "src/main.rs");
    assert!(out.get("content").is_none());
}

#[test]
fn absolute_external_path_is_masked() {
    let out = summarize(
        "read_file",
        &json!({"path": "/Users/kevinh/private.txt"}),
        Path::new("/tmp/project"),
    )
    .unwrap();
    assert_eq!(out["path"], "[external]/private.txt");
}

#[test]
fn bash_secret_command_is_redacted() {
    let out = summarize(
        "bash",
        &json!({"command": "echo $API_KEY"}),
        Path::new("/tmp/project"),
    )
    .unwrap();
    assert_eq!(out["command"], "[redacted command]");
}

#[test]
fn todo_delete_active_arg_is_logged() {
    let out = summarize(
        "todo_delete",
        &json!({"active": true}),
        Path::new("/tmp/project"),
    )
    .unwrap();
    assert_eq!(out["active"], true);
}
