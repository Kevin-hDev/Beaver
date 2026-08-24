use super::*;
use std::path::Path;

#[test]
fn linux_desktop_entry_quotes_spaces_backslashes_and_field_codes() {
    let document = native_entry::documents::linux_desktop(
        ACTIVE_ENTRY_NAME,
        Path::new("/opt/Beaver Folder/100%/beaver\\tool"),
    )
    .unwrap();
    let document = String::from_utf8(document).unwrap();

    assert_eq!(
        document,
        "[Desktop Entry]\nType=Application\nVersion=1.0\nName=Beaver\nExec=\"/opt/Beaver Folder/100%%/beaver\\\\tool\" --clgo-autostart\nStartupNotify=false\nTerminal=false\n"
    );
}

#[test]
fn macos_launch_agent_is_valid_plist_with_exact_arguments() {
    let document = native_entry::documents::macos_launch_agent(
        ACTIVE_ENTRY_NAME,
        Path::new("/Applications/A&B/Beaver.app/Contents/MacOS/Beaver"),
    )
    .unwrap();
    let value = plist::Value::from_reader_xml(document.as_slice()).unwrap();
    let dictionary = value.as_dictionary().unwrap();

    assert_eq!(
        dictionary.get("Label").and_then(plist::Value::as_string),
        Some("Beaver")
    );
    assert_eq!(
        dictionary
            .get("RunAtLoad")
            .and_then(plist::Value::as_boolean),
        Some(true)
    );
    let arguments = dictionary
        .get("ProgramArguments")
        .and_then(plist::Value::as_array)
        .unwrap();
    assert_eq!(
        arguments,
        &[
            plist::Value::String("/Applications/A&B/Beaver.app/Contents/MacOS/Beaver".to_string()),
            plist::Value::String("--clgo-autostart".to_string()),
        ]
    );
}

#[test]
fn native_documents_reject_line_breaks_before_serialization() {
    let invalid = Path::new("/opt/Beaver\nInjected");

    assert!(native_entry::documents::linux_desktop(ACTIVE_ENTRY_NAME, invalid).is_err());
    assert!(native_entry::documents::macos_launch_agent(ACTIVE_ENTRY_NAME, invalid).is_err());
}

#[test]
fn file_entry_reconciles_exact_bytes_and_removes_idempotently() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("autostart").join("Beaver.entry");
    let entry = native_entry::file_entry::FileEntry::new(path.clone(), b"expected\n".to_vec());

    assert_eq!(entry.state().unwrap(), ExactEntryState::Absent);
    synchronize_exact_entry(&entry, true).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"expected\n");

    std::fs::write(&path, b"obsolete\n").unwrap();
    assert_eq!(entry.state().unwrap(), ExactEntryState::Stale);
    synchronize_exact_entry(&entry, true).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"expected\n");

    synchronize_exact_entry(&entry, false).unwrap();
    synchronize_exact_entry(&entry, false).unwrap();
    assert!(!path.exists());
}
