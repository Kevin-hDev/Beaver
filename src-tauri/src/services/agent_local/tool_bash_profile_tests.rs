use super::*;

#[test]
fn known_posix_shells_have_a_snapshot_script() {
    for shell in ["/bin/zsh", "/bin/bash", "/bin/sh"] {
        let script = snapshot_script(shell, "marker").expect("snapshot script");
        assert!(script.contains("marker"));
        assert!(script.contains("export -p"));
    }
    assert!(snapshot_script("/usr/bin/fish", "marker").is_none());
    assert!(!supports_shell("/tmp/custom-zsh-wrapper"));
}

#[cfg(unix)]
#[test]
fn profile_is_kept_out_of_arguments_and_replayed_from_environment() {
    let profile = ShellProfile {
        script: Zeroizing::new(
            "shopt -s expand_aliases; alias hi='printf alias'; myfn() { printf function; }; export BEAVER_PROFILE_TEST=env"
                .to_string(),
        ),
    };
    let command = "hi; myfn; printf '%s:%s' \"$BEAVER_PROFILE_TEST\" \"${BEAVER_INTERNAL_PROFILE_SNAPSHOT-unset}\"";
    let arguments = super::super::tool_bash_shell::shell_arguments(command);

    assert!(arguments
        .iter()
        .all(|argument| !argument.contains("BEAVER_PROFILE_TEST=env")));
    let output = std::process::Command::new("/bin/bash")
        .args(arguments)
        .env(SNAPSHOT_ENV, profile.script.as_str())
        .output()
        .expect("bash");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "aliasfunctionenv:unset");
}
