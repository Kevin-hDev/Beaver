use super::*;

#[test]
#[ignore = "requires a native Windows AppContainer runtime"]
fn appcontainer_writes_only_inside_the_selected_root() {
    let project = tempfile::tempdir().expect("project");
    let outside = tempfile::tempdir().expect("outside");
    let sandbox = tempfile::tempdir().expect("sandbox");
    let project = dunce::canonicalize(project.path()).expect("project canonical");
    let outside_file = outside.path().join("blocked.txt");
    let inside_file = project.join("allowed.txt");
    let executable = super::super::super::tool_bash_platform::powershell_executable()
        .expect("PowerShell");
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; Set-Content -Path '{}' -Value allowed; Set-Content -Path '{}' -Value blocked; if ((Test-Path '{}') -and -not (Test-Path '{}')) {{ exit 0 }} else {{ exit 42 }}",
        escaped(&inside_file),
        escaped(&outside_file),
        escaped(&inside_file),
        escaped(&outside_file),
    );
    let arguments = vec![
        "-NoLogo".into(),
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-Command".into(),
        script.into(),
    ];
    let previous = std::env::current_dir().expect("current dir");
    std::env::set_current_dir(&project).expect("set current dir");
    let result = run(
        &executable,
        &arguments,
        &super::super::scope::Scope::workspace(vec![project.clone()]),
        sandbox.path(),
    );
    std::env::set_current_dir(previous).expect("restore current dir");
    cleanup(sandbox.path());

    assert_eq!(result.expect("AppContainer"), 0);
    assert!(inside_file.is_file());
    assert!(!outside_file.exists());
}

fn escaped(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}
