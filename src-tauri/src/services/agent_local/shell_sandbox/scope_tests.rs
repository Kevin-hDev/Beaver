use super::*;

#[test]
fn profile_capture_accepts_only_direct_regular_profile_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    std::fs::create_dir(&home).expect("home");
    std::fs::write(home.join(".zshrc"), "export PATH=/safe/bin").expect("zshrc");
    std::fs::create_dir(home.join(".bashrc")).expect("directory");
    std::fs::create_dir(home.join(".cargo")).expect("cargo");
    std::fs::write(home.join(".cargo/env"), "export PATH=/cargo/bin:$PATH").expect("cargo env");
    let home = dunce::canonicalize(home).expect("canonical home");

    let files = profile_files_in(&home);

    assert_eq!(files, vec![home.join(".zshrc"), home.join(".cargo/env")]);
}

#[cfg(unix)]
#[test]
fn profile_capture_rejects_a_symlinked_profile() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&home).expect("home");
    std::fs::write(&outside, "secret").expect("outside");
    symlink(&outside, home.join(".zshrc")).expect("symlink");
    let home = dunce::canonicalize(home).expect("canonical home");

    assert!(profile_files_in(&home).is_empty());
}

#[test]
fn profile_capture_allows_only_narrow_tool_and_shell_directories() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    for path in [
        home.join(".nvm"),
        home.join(".local/bin"),
        home.join(".local/share/mise"),
        home.join(".oh-my-zsh/plugins"),
        home.join(".config/zsh/plugins"),
    ] {
        std::fs::create_dir_all(path).expect("profile directory");
    }
    let home = dunce::canonicalize(home).expect("canonical home");

    let directories = profile_dirs_in(&home);

    for expected in [
        home.join(".nvm"),
        home.join(".local/bin"),
        home.join(".local/share/mise"),
        home.join(".oh-my-zsh"),
        home.join(".config/zsh"),
    ] {
        assert!(directories.contains(&expected));
    }
    assert!(!directories.contains(&home));
    assert!(!directories.contains(&home.join(".config")));
    assert!(!directories.contains(&home.join(".local")));
}

#[cfg(unix)]
#[test]
fn profile_capture_rejects_symlinked_tool_directories() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&outside).expect("outside");
    symlink(&outside, home.join(".nvm")).expect("manager symlink");
    let home = dunce::canonicalize(home).expect("canonical home");
    let outside = dunce::canonicalize(outside).expect("canonical outside");

    let directories = profile_dirs_in(&home);

    assert!(!directories.contains(&outside));
}
