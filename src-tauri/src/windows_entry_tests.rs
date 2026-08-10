use super::*;

#[test]
fn development_module_is_refreshed_from_cargo_dependencies() {
    let root = std::env::temp_dir().join(format!(
        "beaver-windows-entry-module-test-{}",
        std::process::id()
    ));
    assert!(root.starts_with(std::env::temp_dir()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir(&root).expect("temporary directory");
    let root = root.canonicalize().expect("canonical temporary directory");
    let dependencies = root.join("deps");
    std::fs::create_dir(&dependencies).expect("dependency directory");
    std::fs::write(root.join("cl_go_dash_lib.dll"), b"stale").expect("stale module");
    std::fs::write(dependencies.join("cl_go_dash_lib.dll"), b"current").expect("current module");

    let module = stage_application_module(&root).expect("staged module");

    assert_eq!(module, root.join("cl_go_dash_lib.dll"));
    assert_eq!(std::fs::read(module).expect("module bytes"), b"current");
    std::fs::remove_dir_all(root).expect("test cleanup");
}

#[test]
fn development_bootstrap_is_staged_with_the_module_basename() {
    let root = tempfile::tempdir().expect("temporary directory");
    let root = root.path().canonicalize().expect("canonical directory");
    let source = root.join("bootstrap.exe");
    std::fs::write(&source, b"verified bootstrap").expect("bootstrap source");

    let executable = stage_bootstrap_executable(&root, &source).expect("staged bootstrap");

    assert_eq!(executable, root.join("cl_go_dash_lib.exe"));
    assert_eq!(
        std::fs::read(executable).expect("bootstrap bytes"),
        b"verified bootstrap"
    );
}

#[test]
fn bootstrap_arguments_only_forward_validated_values() {
    let arguments =
        bootstrap_arguments(vec![OsString::from("--inspect")]).expect("bootstrap arguments");

    assert_eq!(arguments, vec![OsString::from("--inspect")]);
}

#[test]
fn bootstrap_arguments_reject_an_external_module_override() {
    assert!(bootstrap_arguments(vec![OsString::from("--module=other")]).is_err());
}

#[test]
fn bootstrap_role_accepts_only_a_paired_cef_type_and_marker() {
    let parent = classify_bootstrap(vec![OsString::from("beaver.exe")]);
    assert!(matches!(parent, Ok(BootstrapRole::Parent)));

    let helper = classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=secret"),
    ]);
    let Ok(BootstrapRole::CefHelper(marker)) = helper else {
        panic!("valid helper role expected");
    };
    assert_eq!(marker.as_str(), "secret");
}

#[test]
fn bootstrap_role_rejects_unsupervised_or_ambiguous_helpers() {
    assert!(classify_bootstrap(vec![OsString::from("--type=renderer")]).is_err());
    assert!(classify_bootstrap(vec![OsString::from("--beaver-cef-admission=secret")]).is_err());
    assert!(classify_bootstrap(vec![
        OsString::from("--type=renderer"),
        OsString::from("--type=gpu-process"),
        OsString::from("--beaver-cef-admission=secret"),
    ])
    .is_err());
    assert!(classify_bootstrap(vec![
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=first"),
        OsString::from("--beaver-cef-admission=second"),
    ])
    .is_err());
}

#[test]
fn shell_sandbox_process_is_never_classified_as_a_cef_helper() {
    assert!(classify_bootstrap(vec![
        OsString::from("--beaver-shell-sandbox"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=secret"),
    ])
    .is_err());
}

#[test]
fn private_switches_are_classified_case_insensitively() {
    let helper = classify_bootstrap(vec![
        OsString::from("--TYPE=renderer"),
        OsString::from("--BEAVER-CEF-ADMISSION=secret"),
    ]);
    assert!(matches!(helper, Ok(BootstrapRole::CefHelper(_))));
}

#[test]
fn bootstrap_role_is_bounded() {
    let too_many = (0..=MAX_FORWARD_ARGS).map(|_| OsString::from("--safe"));
    assert!(classify_bootstrap(too_many).is_err());
    assert!(classify_bootstrap(vec![OsString::from("x".repeat(MAX_ARG_UTF16 + 1))]).is_err());
}
