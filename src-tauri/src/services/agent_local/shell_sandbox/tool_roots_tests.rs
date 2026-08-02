use super::*;

#[test]
fn tool_roots_add_dependencies_without_broadening_to_home() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let manager = temp.path().join("package-manager");
    let tool = home.join(".hermes/node");
    for path in [
        &home,
        &workspace,
        &manager,
        &tool.join("bin"),
        &home.join("bin"),
        &home.join(".config/git"),
        &home.join(".cargo/registry"),
        &home.join(".cargo/git"),
        &home.join(".npm/_cacache"),
        &home.join(".cache"),
        &home.join(".rustup/toolchains"),
    ] {
        std::fs::create_dir_all(path).expect("create directory");
    }
    for path in [
        home.join(".gitconfig"),
        home.join(".gitignore_global"),
        home.join(".npmrc"),
        home.join(".rustup/settings.toml"),
    ] {
        std::fs::write(path, "test").expect("create file");
    }

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        std::slice::from_ref(&manager),
        Some(&home),
        &[tool.join("bin"), home.join("bin")],
        false,
    );
    let canonical_home = dunce::canonicalize(&home).expect("home");
    let canonical_tool = dunce::canonicalize(&tool).expect("tool");
    let canonical_manager = dunce::canonicalize(&manager).expect("manager");

    assert!(roots.read_dirs.contains(&canonical_manager));
    assert!(roots.read_dirs.contains(&canonical_tool));
    assert!(roots
        .read_dirs
        .iter()
        .all(|root| root != &canonical_home && !canonical_home.starts_with(root)));
    assert!(roots
        .read_dirs
        .iter()
        .all(|root| root.parent().is_some() && root != Path::new("/")));
    let path_dirs = [tool.join("bin"), home.join("bin")]
        .map(|path| dunce::canonicalize(path).expect("path directory"));
    assert!(roots.write_dirs.iter().all(|root| {
        path_dirs
            .iter()
            .all(|path_dir| !path_dir.starts_with(root))
    }));
    assert!(roots.write_files.iter().all(|root| {
        path_dirs
            .iter()
            .all(|path_dir| !root.starts_with(path_dir))
    }));
    assert_eq!(roots.write_dirs.len(), 4);
    assert_eq!(roots.write_files.len(), 1);
}

#[test]
fn executable_path_under_cache_blocks_writable_cache_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let cached_bin = home.join(".cache/tool/bin");
    for path in [&home, &workspace, &cached_bin, &home.join(".cache")] {
        std::fs::create_dir_all(path).expect("create directory");
    }
    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        Some(&home),
        std::slice::from_ref(&cached_bin),
        false,
    );
    let cache = dunce::canonicalize(home.join(".cache")).expect("cache");

    assert!(!roots.write_dirs.contains(&cache));
}

#[test]
fn excessive_path_entries_disable_extra_writable_roots() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(home.join(".cache")).expect("cache");
    std::fs::create_dir_all(&workspace).expect("workspace");

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        Some(&home),
        &[],
        true,
    );

    assert!(roots.write_dirs.is_empty());
    assert!(roots.write_files.is_empty());
}

#[cfg(unix)]
#[test]
fn writable_tool_exceptions_reject_redirecting_symlinks() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let redirected = temp.path().join("redirected");
    let path_dir = redirected.join("bin");
    for path in [&home, &workspace, &path_dir, &home.join(".rustup")] {
        std::fs::create_dir_all(path).expect("directory");
    }
    std::fs::write(path_dir.join("tool"), "binary").expect("tool");
    symlink(&redirected, home.join(".cache")).expect("cache link");
    symlink(
        path_dir.join("tool"),
        home.join(".rustup/settings.toml"),
    )
    .expect("settings link");

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        Some(&home),
        std::slice::from_ref(&path_dir),
        false,
    );

    assert!(!roots
        .write_dirs
        .contains(&dunce::canonicalize(&redirected).unwrap()));
    assert!(roots.write_files.is_empty());
}

#[cfg(unix)]
#[test]
fn current_machine_package_prefixes_are_included_when_present() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    #[cfg(target_os = "macos")]
    let packages = ["/opt/homebrew", "/usr/local"];
    #[cfg(target_os = "linux")]
    let packages = ["/usr/local", "/home/linuxbrew/.linuxbrew"];
    let roots = collect(
        std::slice::from_ref(&workspace),
        &[],
        &packages,
        None,
    );

    for prefix in packages.iter().filter_map(|path| canonical_dir(Path::new(path))) {
        assert!(roots.read_dirs.contains(&prefix));
    }
    assert!(roots.read_dirs.iter().all(|root| root != Path::new("/")));
    if let Some(home) = dirs::home_dir().and_then(|path| canonical_dir(&path)) {
        assert!(!roots.read_dirs.contains(&home));
    }
}
