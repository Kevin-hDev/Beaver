#[cfg(unix)]
use super::*;

#[cfg(unix)]
#[tokio::test]
async fn unrestricted_shell_inherits_the_application_environment() {
    const CHILD_ENV: &str = "BEAVER_ENV_INHERIT_TEST";
    if std::env::var_os(CHILD_ENV).is_some() {
        let arguments = vec![
            "-c".to_string(),
            format!("test \"${{{CHILD_ENV}-}}\" = available"),
        ];
        let mut prepared = prepare_command(
            std::ffi::OsStr::new("/bin/sh"),
            &arguments,
            &std::env::temp_dir(),
        )
        .expect("prepare shell");
        let status = prepared.command.status().await.expect("run shell");
        assert!(status.success());
        return;
    }

    let test_name = concat!(
        "services::agent_local::shell_sandbox::launch::tests::",
        "unrestricted_shell_inherits_the_application_environment"
    );
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_ENV, "available")
        .output()
        .expect("child test");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
