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
        scripts: sanitize::chunks(&sanitize::snapshot(
            "shopt -s expand_aliases; alias hi='printf alias'; myfn() { printf function; }; export PAGER=env\nexport PATH=/short/profile/path",
        )),
    };
    let command = "hi; myfn; printf '%s:%s:%s:%s' \"$PAGER\" \"${BEAVER_INTERNAL_PROFILE_SNAPSHOT_0-unset}\" \"${BEAVER_INTERNAL_PROFILE_SNAPSHOT_1-unset}\" \"$PATH\"";
    let arguments = super::super::tool_bash_shell::shell_arguments(command);

    assert!(arguments
        .iter()
        .all(|argument| !argument.contains("BEAVER_PROFILE_TEST=env")));
    let mut process = std::process::Command::new("/bin/bash");
    process.args(arguments).env("PATH", "/validated/shell/path");
    for (name, script) in SNAPSHOT_ENVS.iter().zip(&profile.scripts) {
        process.env(name, script.as_str());
    }
    let output = process.output().expect("bash");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "aliasfunctionenv:unset:unset:/validated/shell/path"
    );
}

#[test]
fn large_utf8_profiles_are_split_below_linux_environment_limits() {
    let snapshot = "é".repeat(60_000);
    let chunks = sanitize::chunks(&snapshot);

    assert!(chunks
        .iter()
        .all(|chunk| chunk.len() <= sanitize::MAX_SNAPSHOT_CHUNK_BYTES));
    assert_eq!(
        format!("{}{}", chunks[0].as_str(), chunks[1].as_str()),
        snapshot
    );
}

#[test]
fn sandbox_owned_variables_are_not_replayed_from_the_profile() {
    let snapshot = concat!(
        "export TMPDIR=/deleted/sandbox\n",
        "declare -x TMP=\"/deleted/sandbox\"\n",
        " typeset -x TEMP='/deleted/sandbox'\n",
        "export TMPPREFIX=/tmp/zsh\n",
        "export PATH=/short/profile/path\n",
        "export TEMPORARY=kept\n",
        "export PAGER=kept\n",
        "export XDG_CACHE_HOME=/safe/cache\n",
        "export npm_config_cache=/safe/npm\n",
        "export HTTPS_PROXY=http://proxy.example\n",
        "export SSL_CERT_FILE=/corporate/ca.pem\n",
        "export SSH_AUTH_SOCK=/private/ssh-agent.sock\n",
        "export JAVA_HOME=/opt/java\n",
        "export XDG_CONFIG_HOME=/home/user/.config\n",
        "export OPENAI_API_KEY=removed\n",
        "export LD_PRELOAD=/unsafe/injection.so\n",
        "export LD_AUDIT=/unsafe/audit.so\n",
        "export DYLD_INSERT_LIBRARIES=/unsafe/injection.dylib\n",
        "export BEAVER_INTERNAL_SANDBOX_POLICY=removed\n",
    );
    let sanitized = sanitize::snapshot(snapshot);

    assert!(!sanitized.contains("TMPDIR="));
    assert!(!sanitized.contains(" TMP="));
    assert!(!sanitized.contains(" TEMP="));
    assert!(!sanitized.contains("TMPPREFIX="));
    assert!(!sanitized.contains("PATH="));
    assert!(sanitized.contains("TEMPORARY=kept"));
    assert!(sanitized.contains("PAGER=kept"));
    assert!(sanitized.contains("XDG_CACHE_HOME=/safe/cache"));
    assert!(sanitized.contains("npm_config_cache=/safe/npm"));
    assert!(sanitized.contains("HTTPS_PROXY=http://proxy.example"));
    assert!(sanitized.contains("SSL_CERT_FILE=/corporate/ca.pem"));
    assert!(sanitized.contains("SSH_AUTH_SOCK=/private/ssh-agent.sock"));
    assert!(sanitized.contains("JAVA_HOME=/opt/java"));
    assert!(sanitized.contains("XDG_CONFIG_HOME=/home/user/.config"));
    assert!(sanitized.contains("OPENAI_API_KEY=removed"));
    assert!(!sanitized.contains("LD_PRELOAD="));
    assert!(!sanitized.contains("LD_AUDIT="));
    assert!(!sanitized.contains("DYLD_INSERT_LIBRARIES="));
    assert!(!sanitized.contains("BEAVER_INTERNAL_SANDBOX_POLICY="));
}
