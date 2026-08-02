use super::*;
use std::ffi::OsStr;

#[test]
fn path_entries_are_absolute_deduplicated_and_bounded() {
    let mut entries = (0..MAX_PATH_INPUTS + 3)
        .map(|index| format!("/opt/tool-{index}/bin"))
        .collect::<Vec<_>>();
    entries.push("relative/bin".to_string());
    entries.push("/opt/control\n/bin".to_string());
    entries.push("/opt/tool-0/bin".to_string());
    let resolved = normalize(OsString::from(entries.join(":")), true).expect("PATH");

    assert_eq!(resolved.entries.len(), MAX_PATH_INPUTS);
    assert!(resolved.overflow);
    assert!(resolved.entries.iter().all(|entry| entry.is_absolute()));
    assert!(resolved
        .entries
        .iter()
        .all(|entry| !entry.to_string_lossy().chars().any(char::is_control)));
}

#[cfg(unix)]
#[test]
fn login_shell_replaces_a_minimal_gui_path() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let shell = temp.path().join("zsh");
    std::fs::write(
        &shell,
        "#!/bin/sh\nPATH=/opt/beaver-user/bin:/usr/bin:/bin\nexport PATH\nexec /bin/sh -c \"$4\"\n",
    )
    .expect("shell");
    std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700))
        .expect("permissions");

    let captured = unix::capture_for_test(&shell, OsStr::new("/usr/bin:/bin"))
        .expect("captured PATH");
    let resolved = normalize(captured, true).expect("resolved PATH");

    assert!(resolved.discovered);
    assert!(resolved
        .entries
        .contains(&PathBuf::from("/opt/beaver-user/bin")));
}

#[cfg(unix)]
#[test]
fn oversized_login_output_is_rejected() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let shell = temp.path().join("zsh");
    std::fs::write(
        &shell,
        "#!/bin/sh\n/usr/bin/yes x | /usr/bin/head -c 140000\n",
    )
    .expect("shell");
    std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700))
        .expect("permissions");

    assert!(unix::capture_for_test(&shell, OsStr::new("/usr/bin:/bin")).is_none());
}
