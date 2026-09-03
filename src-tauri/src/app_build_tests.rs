use super::AppBuildMode;

#[test]
fn fixture_build_can_run_beside_the_open_application() {
    assert!(AppBuildMode::Interactive.installs_single_instance());
    assert!(!AppBuildMode::LiveFixture.installs_single_instance());
}

#[test]
fn application_build_failure_is_returned_instead_of_panicking() {
    let build = include_str!("app_build.rs");
    let caller = include_str!("lib.rs");

    assert!(build.contains("-> tauri::Result<tauri::App<tauri::Wry>>"));
    assert!(caller.contains("match app_build::build(exit_coordinator, runtime, ui_startup)"));
    assert!(!caller.contains("error while building tauri application"));
}
