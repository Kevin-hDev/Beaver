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
        &home.join(".cache/pip"),
        &home.join(".cache/uv"),
        &home.join(".cache/go-build"),
        &home.join(".cache/yarn"),
        &home.join(".cache/ms-playwright"),
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
    let broad_cache = dunce::canonicalize(home.join(".cache")).expect("cache");
    let executable_cache = dunce::canonicalize(home.join(".cache/ms-playwright"))
        .expect("executable cache");
    assert!(!roots.write_dirs.contains(&broad_cache));
    assert!(!roots.write_dirs.contains(&executable_cache));
    assert_eq!(roots.write_dirs.len(), 7);
    assert_eq!(roots.write_files.len(), 1);
    assert_eq!(MAX_WRITE_ROOTS, 8);
}

#[test]
fn executable_cache_is_not_granted_by_narrow_cache_allowlist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let cached_bin = home.join(".cache/tool/bin");
    for path in [&home, &workspace, &cached_bin, &home.join(".cache/pip")] {
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
    let broad_cache = dunce::canonicalize(home.join(".cache")).expect("cache");
    let safe_cache = dunce::canonicalize(home.join(".cache/pip")).expect("pip cache");

    assert!(!roots.write_dirs.contains(&broad_cache));
    assert!(roots.write_dirs.contains(&safe_cache));
}

#[test]
fn excessive_path_entries_disable_extra_writable_roots() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(home.join(".cache/pip")).expect("cache");
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
    for path in [
        &home,
        &workspace,
        &path_dir,
        &home.join(".rustup"),
        &redirected.join("pip"),
    ] {
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

    assert!(!roots.write_dirs.contains(
        &dunce::canonicalize(redirected.join("pip")).expect("redirected cache")
    ));
    assert!(roots.write_files.is_empty());
}

#[test]
fn read_root_saturation_is_reported_without_exceeding_the_bound() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    let platform = (0..=MAX_READ_ROOTS)
        .map(|index| temp.path().join(format!("read-{index}")))
        .collect::<Vec<_>>();
    for path in &platform {
        std::fs::create_dir(path).expect("read directory");
    }

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &platform,
        &[],
        None,
        &[],
        false,
    );

    assert_eq!(roots.read_dirs.len(), MAX_READ_ROOTS);
    assert!(roots.read_limit_reached);
}

#[cfg(unix)]
#[test]
fn readable_tool_configuration_rejects_symlink_redirection() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let private = home.join(".private");
    for path in [&home.join(".config"), &workspace, &private] {
        std::fs::create_dir_all(path).expect("directory");
    }
    let private_file = private.join("credentials");
    std::fs::write(&private_file, "private").expect("private file");
    symlink(&private_file, home.join(".gitconfig")).expect("config link");
    symlink(&private, home.join(".config/git")).expect("config directory link");

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        Some(&home),
        &[],
        false,
    );
    let private = dunce::canonicalize(private).expect("private directory");
    let private_file = dunce::canonicalize(private_file).expect("private file");

    assert!(!roots.read_dirs.contains(&private));
    assert!(!roots.read_files.contains(&private_file));
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
