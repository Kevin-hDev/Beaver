use super::platform::windows::{fixed_drive_type, valid_destination_text};
use super::platform::windows_cleanup::cleanup_targets;
use super::temp_ownership::{OwnedTempRun, OWNER_MARKER};
use std::fs;

#[cfg(unix)]
#[test]
fn beaver_launch_inherits_the_user_environment() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    let root = std::env::temp_dir().join(format!("beaver-launch-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).unwrap();
    let output = root.join("environment.txt");
    let script = root.join("beaver");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '%s' \"$BEAVER_INSTALLER_LAUNCH_TEST\" > \"$BEAVER_INSTALLER_LAUNCH_OUTPUT\"\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    std::env::set_var("BEAVER_INSTALLER_LAUNCH_TEST", "inherited");
    std::env::set_var("BEAVER_INSTALLER_LAUNCH_OUTPUT", &output);

    super::platform::launch(&script).unwrap();
    for _ in 0..20 {
        if output.is_file() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    std::env::remove_var("BEAVER_INSTALLER_LAUNCH_TEST");
    std::env::remove_var("BEAVER_INSTALLER_LAUNCH_OUTPUT");
    assert_eq!(fs::read_to_string(&output).unwrap(), "inherited");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn windows_destination_accepts_only_a_local_absolute_drive_path() {
    assert!(valid_destination_text(r"C:\Program Files\Beaver"));
    for invalid in [
        r"Beaver",
        r"\\server\share\Beaver",
        r"\\?\C:\Beaver",
        r"\\.\C:\Beaver",
        r"C:\safe\..\escape",
        r"C:\bad*name",
        r"C:\bad?name",
        r#"C:\bad"name"#,
        "C:\\bad\nname",
        r"C:\CON",
        r"C:\name. ",
        r"C:\extra:stream",
    ] {
        assert!(!valid_destination_text(invalid), "accepted {invalid:?}");
    }
    assert!(!valid_destination_text(&format!(
        "C:\\{}",
        "a".repeat(1_024)
    )));
}

#[test]
fn windows_destination_requires_a_fixed_drive() {
    assert!(fixed_drive_type(3));
    for denied in [0, 1, 2, 4, 5, 6] {
        assert!(!fixed_drive_type(denied));
    }
}

#[test]
fn windows_cleanup_orders_the_executable_before_its_owned_directory() {
    let id = "0123456789abcdef0123456789abcdef";
    let root = std::env::temp_dir().join(format!("beaver-cleanup-order-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).unwrap();
    let run_path = root.join(format!("beaver-install-{id}"));
    fs::create_dir(&run_path).unwrap();
    fs::write(
        run_path.join(OWNER_MARKER),
        format!(r#"{{"schema":1,"runId":"{id}"}}"#),
    )
    .unwrap();
    let executable = run_path.join("beaver-installer.exe");
    fs::write(&executable, "binary").unwrap();
    let run = OwnedTempRun::adopt(&root, &run_path, id).unwrap();
    let targets = cleanup_targets(&run, &executable).unwrap();
    assert_eq!(targets[0], executable.canonicalize().unwrap());
    assert_eq!(targets[1], run_path.canonicalize().unwrap());
    drop(run);
    fs::remove_dir_all(root).unwrap();
}
