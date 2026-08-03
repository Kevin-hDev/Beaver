use super::*;

const CHILD_ENV: &str = "BEAVER_LINUX_SANDBOX_CHILD";

#[test]
fn landlock_enforcement_status_fails_closed() {
    assert!(!isolation_is_unavailable(&RulesetStatus::FullyEnforced));
    assert!(isolation_is_unavailable(
        &RulesetStatus::PartiallyEnforced
    ));
    assert!(isolation_is_unavailable(&RulesetStatus::NotEnforced));
}

#[test]
#[ignore = "requires a native Linux Landlock runtime"]
fn landlock_writes_only_inside_the_selected_root() {
    if let Some(specification) = std::env::var_os(CHILD_ENV) {
        let [project, outside, sandbox] = std::env::split_paths(&specification)
            .collect::<Vec<_>>()
            .try_into()
            .expect("three paths");
        let project = dunce::canonicalize(project).expect("project");
        let outside = dunce::canonicalize(outside).expect("outside");
        let sandbox = dunce::canonicalize(sandbox).expect("sandbox");
        let outside_file = outside.join("blocked.txt");
        let script = "mkdir \"$1/mounted\"; if mount --bind / \"$1/mounted\" 2>/dev/null; then exit 43; fi; printf allowed > \"$1/allowed.txt\"; printf blocked > \"$2\" 2>/dev/null || true; test -f \"$1/allowed.txt\" && test ! -e \"$2\"";
        let arguments = vec![
            "-c".into(),
            script.into(),
            "beaver-test".into(),
            project.clone().into_os_string(),
            outside_file.into_os_string(),
        ];
        let code = run(
            Path::new("/bin/sh"),
            &arguments,
            &super::super::scope::Scope::workspace(vec![project]),
            &sandbox,
        )
        .expect("Landlock");
        std::process::exit(code);
    }

    let test_root = tempfile::tempdir_in(std::env::current_dir().expect("current directory"))
        .expect("test root");
    let project = test_root.path().join("project");
    let outside = test_root.path().join("outside");
    let sandbox = test_root.path().join("sandbox");
    std::fs::create_dir(&project).expect("project");
    std::fs::create_dir(&outside).expect("outside");
    std::fs::create_dir(&sandbox).expect("sandbox");
    let specification =
        std::env::join_paths([&project, &outside, &sandbox]).expect("path list");
    let test_name = concat!(
        "services::agent_local::shell_sandbox::linux::tests::",
        "landlock_writes_only_inside_the_selected_root"
    );
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--ignored", "--nocapture"])
        .env(CHILD_ENV, specification)
        .output()
        .expect("child test");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(project.join("allowed.txt").is_file());
    assert!(!outside.join("blocked.txt").exists());
}
