use std::process::Command;

#[test]
fn updater_failure_is_visible_before_tauri_logging_exists() {
    let output = Command::new(env!("CARGO_BIN_EXE_cl-go-dash-updater"))
        .output()
        .expect("launch updater without arguments");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "update failed"
    );
}
