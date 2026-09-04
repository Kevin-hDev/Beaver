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
fn parent_captures_safe_startup_before_any_cef_execution() {
    let source = include_str!("windows_entry.rs");
    let parent = source.find("fn run_parent(").expect("parent entry");
    let prepare = source[parent..]
        .find("prepare_ui_startup()")
        .expect("safe startup capture");
    let cef = source[parent..]
        .find("execute_process(")
        .expect("CEF process execution");
    let transported = source[parent..]
        .find("run_windows_with_ui_startup(ui_startup)")
        .expect("transported startup state");

    assert!(prepare < cef);
    assert!(cef < transported);
}
