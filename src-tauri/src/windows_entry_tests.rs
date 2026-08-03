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
fn bootstrap_arguments_select_the_application_module_before_forwarded_values() {
    let arguments =
        bootstrap_arguments(vec![OsString::from("--inspect")]).expect("bootstrap arguments");

    assert_eq!(
        arguments,
        vec![
            OsString::from("--module=cl_go_dash_lib"),
            OsString::from("--inspect"),
        ]
    );
}

#[test]
fn bootstrap_arguments_reject_an_external_module_override() {
    assert!(bootstrap_arguments(vec![OsString::from("--module=other")]).is_err());
}
