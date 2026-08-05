use super::*;
#[cfg(unix)]
use std::ffi::OsStr;

#[test]
fn path_entries_are_absolute_deduplicated_and_bounded() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut entries = (0..MAX_PATH_INPUTS + 3)
        .map(|index| temp.path().join(format!("tool-{index}/bin")))
        .collect::<Vec<_>>();
    entries.push(PathBuf::from("relative/bin"));
    entries.push(temp.path().join("control\n/bin"));
    entries.push(temp.path().join("tool-0/bin"));
    let value = std::env::join_paths(&entries).expect("join PATH");
    let resolved = normalize(value, false).expect("PATH");

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
    let user_bin = temp.path().join("beaver-user/bin");
    std::fs::create_dir_all(&user_bin).expect("user bin");
    std::fs::write(
        &shell,
        format!(
            "#!/bin/sh\nPATH={}:$PATH\nexport PATH\nexec /bin/sh -c \"$4\"\n",
            user_bin.display()
        ),
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
        .contains(&user_bin));
}

#[test]
fn discovered_path_drops_entries_that_cannot_provide_tools() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");
    let available = temp.path().join("available");
    std::fs::create_dir(&available).expect("available PATH entry");
    let value = std::env::join_paths([missing.as_path(), available.as_path()]).expect("join PATH");
    let resolved = normalize(value, true).expect("PATH");

    assert!(!resolved.entries.contains(&missing));
    assert!(resolved.entries.contains(&available));
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

#[cfg(unix)]
#[test]
fn login_path_is_refined_with_the_first_captured_entries() {
    let temp = tempfile::tempdir().expect("tempdir");
    let local_bin = temp.path().join("home/.local/bin");
    let nvm_bin = temp.path().join("home/.nvm/current/bin");
    std::fs::create_dir_all(&local_bin).expect("local bin");
    std::fs::create_dir_all(&nvm_bin).expect("nvm bin");
    let first_path = std::env::join_paths([local_bin.as_path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("first PATH");
    let refined_path = std::env::join_paths([
        nvm_bin.as_path(),
        local_bin.as_path(),
        Path::new("/usr/bin"),
        Path::new("/bin"),
    ])
    .expect("refined PATH");
    let mut inputs = Vec::new();
    let captured = unix::refine_for_test(OsStr::new("/usr/bin:/bin"), |path, _| {
        inputs.push(path.to_os_string());
        if inputs.len() == 1 {
            Some(first_path.clone())
        } else {
            Some(refined_path.clone())
        }
    })
    .expect("refined PATH");

    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[1], first_path);
    assert_eq!(captured, refined_path);
}

#[cfg(unix)]
#[test]
fn first_captured_path_survives_a_failed_refinement() {
    let temp = tempfile::tempdir().expect("tempdir");
    let custom_bin = temp.path().join("custom/bin");
    std::fs::create_dir_all(&custom_bin).expect("custom bin");
    let first_path = std::env::join_paths([
        custom_bin.as_path(),
        Path::new("/usr/bin"),
        Path::new("/bin"),
    ])
    .expect("first PATH");
    let mut calls = 0;
    let captured = unix::refine_for_test(OsStr::new("/usr/bin:/bin"), |_, _| {
        calls += 1;
        (calls == 1).then(|| first_path.clone())
    })
    .expect("first PATH");

    assert_eq!(calls, 2);
    assert_eq!(captured, first_path);
}
