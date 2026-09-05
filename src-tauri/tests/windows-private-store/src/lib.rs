#![cfg(windows)]
#![allow(dead_code)]

#[path = "../../../src/services/windows_fs_retry.rs"]
mod windows_fs_retry;

mod services {
    // Mirror the production module path so this isolated crate compiles the real implementation.
    pub(crate) use crate::windows_fs_retry;

    pub mod paths {
        pub fn data_dir() -> std::path::PathBuf {
            std::env::temp_dir().join("cl-go-private-store-integration")
        }
    }

    // This isolated crate exercises the Windows ACL implementation only. The
    // application-owned debug fixture cleanup is covered by its own test suite.
    pub mod reasoning_fixture_tools {
        pub fn purge_stale_runtime() -> Result<(), String> {
            Ok(())
        }
    }
}

#[path = "../../../src/services/private_store.rs"]
mod private_store;

#[test]
fn private_storage_round_trip_uses_the_windows_acl_implementation() {
    use rand::RngCore;

    let mut random = [0_u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut random);
    let root = std::env::temp_dir().join(format!(
        "cl-go-private-store-integration-{}",
        hex::encode(random)
    ));
    let path = root.join("nested/private.json");

    private_store::atomic_write(&path, b"one").expect("first private write");
    private_store::atomic_write(&path, b"two").expect("second private write");
    private_store::repair_path(&root).expect("directory ACL repair");
    private_store::repair_path(&path).expect("file ACL repair");

    assert_eq!(std::fs::read(&path).expect("private file"), b"two");
    assert_eq!(
        std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
        1
    );
    let _ = std::fs::remove_dir_all(root);
}
