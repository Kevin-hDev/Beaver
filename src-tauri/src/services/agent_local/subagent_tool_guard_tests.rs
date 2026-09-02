use super::subagent_explorer_bash;
use super::subagent_tool_guard;
use super::subagent_tool_profile::SubagentToolProfile;
use serde_json::json;
use crate::services::extensions::ExtensionEffect;

#[test]
fn explorer_bash_accepts_only_informational_commands() {
    let root = tempfile::tempdir().expect("root");
    for command in [
        "pwd",
        "ls -la .",
        "tree -L 3 .",
        "file Cargo.toml",
        "stat Cargo.toml",
        "wc -l Cargo.toml",
        "du -sh .",
        "df -h .",
        "git status --short",
        "git diff --stat",
        "git log -5",
        "git show HEAD",
        "git rev-parse HEAD",
        "git ls-files",
        "git remote -v",
        "git tag --list",
        "git branch",
    ] {
        assert!(
            subagent_explorer_bash::validate(command, root.path()).is_ok(),
            "commande refusée: {command}"
        );
    }
}

#[test]
fn explorer_bash_rejects_shell_network_mutations_and_escape() {
    let root = tempfile::tempdir().expect("root");
    for command in [
        "tree -L 0",
        "tree -L 9",
        "find . -type f",
        "ls | wc -l",
        "ls && pwd",
        "ls > out.txt",
        "echo $(pwd)",
        "curl https://example.com",
        "git checkout main",
        "git branch new-name",
        "git branch --delete",
        "ls ..",
        "stat /etc/passwd",
    ] {
        assert!(
            subagent_explorer_bash::validate(command, root.path()).is_err(),
            "commande acceptée: {command}"
        );
    }
}

#[test]
fn file_tools_and_coder_bash_stay_in_worktree() {
    let root = tempfile::tempdir().expect("root");
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).expect("nested");
    let inside = root.path().join("inside.txt");
    std::fs::write(&inside, "ok").expect("inside");
    let outside = tempfile::NamedTempFile::new().expect("outside");

    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "read_file",
        &json!({"path": "inside.txt"}),
        root.path(),
    )
    .is_ok());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "read_file",
        &json!({"path": outside.path()}),
        root.path(),
    )
    .is_err());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "write_file",
        &json!({"path": "../outside.txt"}),
        root.path(),
    )
    .is_err());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "bash",
        &json!({"command": "cargo test"}),
        root.path(),
    )
    .is_ok());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "bash",
        &json!({"command": "cargo test", "workdir": nested}),
        root.path(),
    )
    .is_ok());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "bash",
        &json!({"command": "cargo test", "workdir": outside.path()}),
        root.path(),
    )
    .is_err());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "bash",
        &json!({"command": format!("git -C {} status", outside.path().display())}),
        root.path(),
    )
    .is_err());
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        true,
        "bash",
        &json!({"command": format!("cargo test >{}", outside.path().display())}),
        root.path(),
    )
    .is_err());
}

#[test]
fn subagents_can_only_read_an_explicit_memory_file() {
    let root = tempfile::tempdir().expect("root");
    let topic = crate::services::paths::data_dir()
        .join("memory/global/topics/00000000-0000-4000-8000-000000000000.md");

    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Explorer,
        false,
        "read_file",
        &json!({"path": topic}),
        root.path(),
    )
    .is_ok());
    for tool_name in ["list_dir", "grep", "glob"] {
        assert!(subagent_tool_guard::validate_for_profile(
            SubagentToolProfile::Explorer,
            false,
            tool_name,
            &json!({"path": topic}),
            root.path(),
        )
        .is_err());
    }
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Coder,
        false,
        "write_file",
        &json!({"path": topic, "content": "contenu"}),
        root.path(),
    )
    .is_err());
}

#[test]
fn an_invalid_memory_path_is_refused_instead_of_bypassing_confinement() {
    let root = tempfile::tempdir().expect("root");

    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Explorer,
        false,
        "read_file",
        &json!({"path": "\0"}),
        root.path(),
    )
    .is_err());
}

#[cfg(unix)]
#[test]
fn outgoing_symlink_is_rejected() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().expect("root");
    let outside = tempfile::NamedTempFile::new().expect("outside");
    symlink(outside.path(), root.path().join("link")).expect("symlink");
    assert!(subagent_tool_guard::validate_for_profile(
        SubagentToolProfile::Explorer,
        false,
        "read_file",
        &json!({"path": "link"}),
        root.path(),
    )
    .is_err());
}

#[test]
fn child_extension_access_uses_the_parent_mode_and_exact_cache() {
    use super::subagent_tool_guard::ChildExtensionDecision::{Allow, Deny};
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Explorer,
            ExtensionEffect::ReadOnly,
            "manual",
            false,
        ),
        Allow,
    );
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Coder,
            ExtensionEffect::ReadOnly,
            "manual",
            false,
        ),
        Allow,
    );
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Explorer,
            ExtensionEffect::Secret,
            "auto",
            true,
        ),
        Deny,
    );
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Coder,
            ExtensionEffect::Secret,
            "auto",
            false,
        ),
        Allow,
    );
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Coder,
            ExtensionEffect::Secret,
            "manual",
            false,
        ),
        Deny,
    );
    assert_eq!(
        super::subagent_tool_guard::child_extension_decision(
            SubagentToolProfile::Coder,
            ExtensionEffect::Secret,
            "manual",
            true,
        ),
        Allow,
    );
}
