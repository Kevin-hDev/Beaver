use super::*;
use std::ffi::{OsStr, OsString};
use std::process::Command;

fn environment(command: &Command) -> std::collections::HashMap<OsString, Option<OsString>> {
    command
        .get_envs()
        .map(|(key, value)| (key.to_owned(), value.map(OsStr::to_owned)))
        .collect()
}

#[test]
fn host_profile_clears_secrets_and_keeps_only_runtime_variables() {
    let temporary = tempfile::tempdir().unwrap();
    let mut command = tokio::process::Command::new("unused");
    command.env("BEAVER_SECRET_SENTINEL", "must-not-leak");

    configure_host(&mut command, OsString::from("safe-path"), temporary.path()).unwrap();

    let values = environment(command.as_std());
    assert!(!values.contains_key(OsStr::new("BEAVER_SECRET_SENTINEL")));
    assert!(!values.contains_key(OsStr::new("HOME")));
    assert_eq!(
        values.get(OsStr::new("TMPDIR")),
        Some(&Some(temporary.path().as_os_str().to_owned()))
    );
}

#[test]
fn installer_profile_adds_an_isolated_home_without_restoring_secrets() {
    let temporary = tempfile::tempdir().unwrap();
    let mut command = Command::new("unused");
    command.env("BEAVER_SECRET_SENTINEL", "must-not-leak");

    configure_installer(&mut command, OsString::from("safe-path"), temporary.path()).unwrap();

    let values = environment(&command);
    assert!(!values.contains_key(OsStr::new("BEAVER_SECRET_SENTINEL")));
    assert_eq!(
        values.get(OsStr::new("HOME")),
        Some(&Some(temporary.path().as_os_str().to_owned()))
    );
}

#[test]
fn real_installer_child_cannot_read_a_preconfigured_secret_sentinel() {
    let temporary = tempfile::tempdir().unwrap();
    let mut command = Command::new(which::which("node").unwrap());
    command
        .args([
            "-e",
            "process.stdout.write(JSON.stringify({secret:process.env.BEAVER_SECRET_SENTINEL,home:process.env.HOME,tmp:process.env.TMPDIR}))",
        ])
        .env("BEAVER_SECRET_SENTINEL", "must-not-leak");
    configure_installer(
        &mut command,
        OsString::from("/usr/bin:/bin"),
        temporary.path(),
    )
    .unwrap();

    let output = command.output().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.get("secret").is_none());
    assert_eq!(value["home"], temporary.path().to_string_lossy().as_ref());
    assert_eq!(value["tmp"], temporary.path().to_string_lossy().as_ref());
}

#[tokio::test]
async fn real_host_child_cannot_read_secret_or_home_from_beaver() {
    let temporary = tempfile::tempdir().unwrap();
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command
        .args([
            "-e",
            "process.stdout.write(JSON.stringify({secret:process.env.BEAVER_SECRET_SENTINEL,home:process.env.HOME,tmp:process.env.TMPDIR}))",
        ])
        .env("BEAVER_SECRET_SENTINEL", "must-not-leak");
    configure_host(
        &mut command,
        OsString::from("/usr/bin:/bin"),
        temporary.path(),
    )
    .unwrap();

    let output = command.output().await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.get("secret").is_none());
    assert!(value.get("home").is_none());
    assert_eq!(value["tmp"], temporary.path().to_string_lossy().as_ref());
}
