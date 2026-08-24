use super::*;
use std::ffi::OsString;
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_BINARY};
use winreg::{RegKey, RegValue};

const RUN_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
const APPROVED_KEY: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

struct RegistryCleanup(String);

impl Drop for RegistryCleanup {
    fn drop(&mut self) {
        if let Ok(key) =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        {
            let _ = key.delete_value(&self.0);
        }
        if let Ok(key) =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED_KEY, KEY_SET_VALUE)
        {
            let _ = key.delete_value(&self.0);
        }
    }
}

fn registry_value(name: &str) -> Option<OsString> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ)
        .ok()?
        .get_value(name)
        .ok()
}

fn set_registry_value(name: &str, value: &str) {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .unwrap()
        .set_value(name, &value)
        .unwrap();
}

fn set_approval(name: &str, bytes: &[u8]) {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(APPROVED_KEY)
        .unwrap();
    key.set_raw_value(
        name,
        &RegValue {
            vtype: REG_BINARY,
            bytes: bytes.to_vec(),
        },
    )
    .unwrap();
}

fn approval(name: &str) -> Option<Vec<u8>> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(APPROVED_KEY, KEY_READ)
        .ok()?
        .get_raw_value(name)
        .ok()
        .map(|value| value.bytes)
}

#[test]
fn windows_entry_quotes_repairs_and_removes_the_exact_command() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("folder with space").join("Beaver.exe");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    std::fs::write(&executable, b"test executable").unwrap();
    let name = format!("Beaver-Autostart-Test-{}", uuid::Uuid::new_v4());
    let _cleanup = RegistryCleanup(name.clone());
    let entry = native_entry::NativeEntry::new(&name, Path::new(&executable)).unwrap();
    let expected = OsString::from(format!("\"{}\" --clgo-autostart", executable.display()));

    synchronize_exact_entry(&entry, true).unwrap();
    assert_eq!(registry_value(&name), Some(expected.clone()));

    set_registry_value(&name, "C:\\obsolete\\Beaver.exe");
    assert_eq!(entry.state().unwrap(), ExactEntryState::Stale);
    synchronize_exact_entry(&entry, true).unwrap();
    assert_eq!(registry_value(&name), Some(expected));

    set_approval(&name, &[3, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(entry.state().unwrap(), ExactEntryState::Stale);
    synchronize_exact_entry(&entry, true).unwrap();
    assert_eq!(
        approval(&name),
        Some(vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );

    synchronize_exact_entry(&entry, false).unwrap();
    synchronize_exact_entry(&entry, false).unwrap();
    assert_eq!(registry_value(&name), None);
    assert_eq!(approval(&name), None);
}
