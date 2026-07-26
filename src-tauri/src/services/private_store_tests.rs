use super::atomic_write;
#[cfg(any(unix, windows))]
use super::repair_path;
use rand::RngCore;

fn test_dir() -> std::path::PathBuf {
    let mut random = [0_u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut random);
    std::env::temp_dir().join(format!("cl-go-private-{}", hex::encode(random)))
}

#[cfg(unix)]
#[test]
fn atomic_write_creates_private_directories_and_files() {
    use std::os::unix::fs::PermissionsExt;

    let root = test_dir();
    let path = root.join("nested/secret.json");
    atomic_write(&path, b"secret").unwrap();
    assert_eq!(
        std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(root.join("nested"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"secret");
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn repair_path_removes_existing_group_and_world_access() {
    use std::os::unix::fs::PermissionsExt;

    let root = test_dir();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("private.json");
    std::fs::write(&path, b"data").unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    repair_path(&root).unwrap();
    repair_path(&path).unwrap();
    assert_eq!(
        std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn app_storage_repairs_the_forecast_notes_directory() {
    use std::os::unix::fs::PermissionsExt;

    super::repair_app_storage().unwrap();
    let path = crate::services::paths::data_dir().join("forecast-notes");
    let mode = std::fs::metadata(path).unwrap().permissions().mode();

    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn atomic_write_leaves_no_temporary_file() {
    let root = test_dir();
    let path = root.join("private.json");
    atomic_write(&path, b"one").unwrap();
    atomic_write(&path, b"two").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"two");
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn windows_acl_implementation_uses_no_external_commands() {
    let sources = [
        include_str!("private_store/private_store_windows.rs"),
        include_str!("private_store/windows_acl.rs"),
        include_str!("private_store/windows_token.rs"),
    ];
    for source in sources {
        for forbidden in ["std::process::Command", "Command::new", "icacls", "whoami"] {
            assert!(
                !source.contains(forbidden),
                "Windows ACL implementation must not depend on {forbidden}"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn repair_path_is_repeatable_for_current_user_directory() {
    let root = test_dir();
    std::fs::create_dir_all(&root).unwrap();

    repair_path(&root).unwrap();
    repair_path(&root).unwrap();
    std::fs::write(root.join("probe.txt"), b"ok").unwrap();
    assert_eq!(std::fs::read(root.join("probe.txt")).unwrap(), b"ok");

    let _ = std::fs::remove_dir_all(root);
}
