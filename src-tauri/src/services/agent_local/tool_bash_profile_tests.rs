use super::*;

#[test]
fn quoted_commands_preserve_quotes_and_newlines() {
    let profile = ShellProfile {
        script: Zeroizing::new("alias hi='printf hello'".to_string()),
    };
    let wrapped = profile.wrap("printf '%s\\n' \"a'b\"\nprintf done");

    assert!(wrapped.starts_with("alias hi="));
    assert!(wrapped.contains("'\"'\"'"));
    assert!(wrapped.ends_with('\''));
}

#[test]
fn known_posix_shells_have_a_snapshot_script() {
    for shell in ["/bin/zsh", "/bin/bash", "/bin/sh"] {
        let script = snapshot_script(shell, "marker").expect("snapshot script");
        assert!(script.contains("marker"));
        assert!(script.contains("export -p"));
    }
    assert!(snapshot_script("/usr/bin/fish", "marker").is_none());
}

#[cfg(unix)]
#[test]
fn wrapped_command_replays_aliases_functions_and_exports() {
    let profile = ShellProfile {
        script: Zeroizing::new(
            "shopt -s expand_aliases; alias hi='printf alias'; myfn() { printf function; }; export BEAVER_PROFILE_TEST=env"
                .to_string(),
        ),
    };
    let wrapped = profile.wrap("hi; myfn; printf %s \"$BEAVER_PROFILE_TEST\"");
    let output = std::process::Command::new("/bin/bash")
        .args(["-c", wrapped.as_str()])
        .output()
        .expect("bash");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "aliasfunctionenv");
}
