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

#[cfg(target_os = "macos")]
#[test]
fn unrestricted_macos_parent_guard_routes_through_helper() {
    let source = include_str!("launch.rs");
    assert!(source.contains("return super::macos_parent_guard::prepare"));
    let prepared = super::super::macos_parent_guard::guarded_command_for_test(
        std::ffi::OsStr::new("/bin/sh"),
        &["-c".to_string(), "true".to_string()],
        std::ffi::OsStr::new("/usr/bin:/bin"),
    )
    .expect("prepare guarded shell");
    let command = prepared.command.as_std();
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        command.get_program(),
        std::env::current_exe().expect("test executable")
    );
    assert_eq!(
        args.first().map(String::as_str),
        Some("--beaver-shell-guard")
    );
    assert_eq!(args.get(1).map(String::as_str), Some("--"));
    assert_eq!(args.get(2).map(String::as_str), Some("/bin/sh"));
}
