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
        &home.join("go/pkg/mod"),
        &home.join("go/pkg/sumdb"),
        &home.join(".cache/ms-playwright"),
        &home.join(".rustup/toolchains"),
    ] {
        std::fs::create_dir_all(path).expect("create directory");
    }
    make_executable(&tool.join("bin/tool"));
    make_executable(&home.join("bin/tool"));
    for path in [
        home.join(".gitconfig"),
        home.join(".gitignore_global"),
        home.join(".npmrc"),
        home.join(".rustup/settings.toml"),
    ] {
        std::fs::write(path, "test").expect("create file");
    }
    let writable_cache_dirs = [
        home.join(".cargo/registry"),
        home.join(".cargo/git"),
        home.join(".npm/_cacache"),
        home.join(".cache/pip"),
        home.join(".cache/uv"),
        home.join(".cache/go-build"),
        home.join(".cache/yarn"),
        home.join("go/pkg/mod"),
        home.join("go/pkg/sumdb"),
    ];

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        std::slice::from_ref(&manager),
        Some(&home),
        &[tool.join("bin"), home.join("bin")],
        false,
        &writable_cache_dirs,
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
    assert_eq!(roots.write_dirs.len(), 9);
    assert_eq!(roots.write_files.len(), 1);
    let source_home = temp.path().join("source-home");
    std::fs::create_dir_all(source_home.join(".openclaw/workspace-one"))
        .expect("openclaw workspace one");
    std::fs::create_dir_all(source_home.join(".openclaw/workspace-two"))
        .expect("openclaw workspace two");
    let (source_dirs, source_files) =
        crate::services::agent_import::declared_resource_counts(&source_home);
    let worst_case = source_dirs
        + source_files
        + super::super::super::agent_resource_access::LOCAL_RESOURCE_DIRS
        + super::super::super::agent_resource_access::LOCAL_RESOURCE_FILES
        + super::super::tool_cache_roots::MAX_WRITE_DIRS
        + 1;
    assert!(MAX_WRITE_ROOTS >= worst_case, "required: {worst_case}");
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
        &[home.join(".cache/pip")],
    );
    let broad_cache = dunce::canonicalize(home.join(".cache")).expect("cache");
    let safe_cache = dunce::canonicalize(home.join(".cache/pip")).expect("pip cache");

    assert!(!roots.write_dirs.contains(&broad_cache));
    assert!(roots.write_dirs.contains(&safe_cache));
}

#[test]
fn configured_cache_paths_stay_inside_home_and_never_cover_path_tools() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let workspace = temp.path().join("workspace");
    let safe_cache = home.join("custom/safe-cache");
    let executable_cache = home.join("custom/executable-cache");
    let executable_dir = executable_cache.join("bin");
    let external_cache = temp.path().join("external-cache");
    for path in [
        &home,
        &workspace,
        &safe_cache,
        &executable_dir,
        &external_cache,
    ] {
        std::fs::create_dir_all(path).expect("directory");
    }
    make_executable(&executable_dir.join("tool"));

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        Some(&home),
        std::slice::from_ref(&executable_dir),
        false,
        &[
            safe_cache.clone(),
            executable_cache.clone(),
            external_cache.clone(),
        ],
    );
    let safe_cache = dunce::canonicalize(safe_cache).expect("safe cache");
    let executable_cache = dunce::canonicalize(executable_cache).expect("executable cache");
    let external_cache = dunce::canonicalize(external_cache).expect("external cache");

    assert!(roots.write_dirs.contains(&safe_cache));
    assert!(!roots.write_dirs.contains(&executable_cache));
    assert!(!roots.write_dirs.contains(&external_cache));
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
        &[home.join(".cache/pip")],
    );

    assert!(roots.write_dirs.is_empty());
    assert!(roots.write_files.is_empty());
}

#[test]
fn read_only_collection_never_returns_cache_writes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");

    let roots = collect_read_only(std::slice::from_ref(&workspace), &[], &[], None);

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
    make_executable(&path_dir.join("tool"));
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
        &[home.join(".cache/pip")],
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
        &[],
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
        &[],
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

#[cfg(unix)]
#[test]
fn path_directory_without_an_executable_is_not_granted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let sensitive = temp.path().join("sensitive");
    std::fs::create_dir(&workspace).expect("workspace");
    std::fs::create_dir(&sensitive).expect("sensitive");
    std::fs::write(sensitive.join("credentials"), "secret").expect("credentials");

    let roots = collect_from(
        std::slice::from_ref(&workspace),
        &[],
        &[],
        None,
        std::slice::from_ref(&sensitive),
        false,
        &[],
    );

    assert!(!roots
        .read_dirs
        .contains(&dunce::canonicalize(sensitive).expect("sensitive")));
}

#[test]
fn path_parent_never_exposes_the_private_application_store() {
    let private_store = crate::services::paths::data_dir();
    let private_store = dunce::canonicalize(private_store).expect("private store");

    for ancestor in private_store.ancestors().skip(1) {
        assert!(super::super::tool_roots_path::forbidden_broad_root(ancestor, None));
    }
}

#[test]
fn private_store_is_read_only_unless_a_configured_root_covers_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let private = temp.path().join("private");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&private).expect("private");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let private = dunce::canonicalize(private).expect("private");
    let workspace = dunce::canonicalize(workspace).expect("workspace");
    let mut restricted = ToolRoots::default();

    push_private_read_dir(
        &mut restricted,
        &private,
        std::slice::from_ref(&workspace),
    );

    assert!(restricted.read_dirs.contains(&private));
    assert!(!restricted.write_dirs.contains(&private));

    let mut explicitly_allowed = ToolRoots::default();
    push_private_read_dir(
        &mut explicitly_allowed,
        &private,
        std::slice::from_ref(&private),
    );
    assert!(!explicitly_allowed.read_dirs.contains(&private));
}

#[cfg(unix)]
#[test]
fn private_store_read_root_rejects_a_symbolic_link() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let target = temp.path().join("target");
    let redirected = temp.path().join("private");
    std::fs::create_dir(&target).expect("target");
    symlink(&target, &redirected).expect("symlink");
    let mut roots = ToolRoots::default();

    push_private_read_dir(&mut roots, &redirected, &[]);

    assert!(roots.read_dirs.is_empty());
}

#[test]
fn workspace_collection_always_reads_the_private_store() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");

    let roots = collect(std::slice::from_ref(&workspace), &[], &[], None);
    let private = dunce::canonicalize(crate::services::paths::data_dir()).expect("private");

    assert!(roots.read_dirs.contains(&private));
    assert!(!roots.write_dirs.contains(&private));
}

#[test]
fn enabled_agent_resources_are_writable_without_opening_their_parent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let source = temp.path().join("source");
    let rules = source.join("rules");
    let document = source.join("AGENTS.md");
    std::fs::create_dir_all(&workspace).expect("workspace");
    std::fs::create_dir_all(&rules).expect("rules");
    std::fs::write(&document, "rules").expect("document");
    let mut roots = ToolRoots::default();

    append_resource_access(
        &mut roots,
        std::slice::from_ref(&workspace),
        &[],
        super::super::super::agent_resource_access::AgentResourceAccess {
            directories: vec![rules.clone()],
            files: vec![document.clone()],
        },
    );
    let rules = dunce::canonicalize(rules).expect("canonical rules");
    let document = dunce::canonicalize(document).expect("canonical document");
    let source = dunce::canonicalize(source).expect("canonical source");

    assert!(roots.write_dirs.contains(&rules));
    assert!(roots.write_files.contains(&document));
    assert!(!roots.write_dirs.contains(&source));
}

#[test]
fn saturated_root_limit_is_silent_and_persists_a_bounded_diagnostic() {
    const CHILD_PATH: &str = "BEAVER_ROOT_LIMIT_TEST_PATH";
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("workspace");
        let platform = (0..=MAX_READ_ROOTS)
            .map(|index| temp.path().join(format!("tool-{index}")))
            .collect::<Vec<_>>();
        for path in &platform {
            std::fs::create_dir_all(path).expect("tool root");
        }
        let roots = collect_from(
            std::slice::from_ref(&workspace),
            &platform,
            &[],
            None,
            &[],
            false,
            &[],
        );
        assert!(roots.read_limit_reached);
        super::super::super::shell_diagnostics::record_root_limits_for_test(
            false,
            roots.read_limit_reached,
            roots.write_limit_reached,
            Path::new(&path),
        );
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let diagnostic = temp.path().join("root-limit.json");
    let test_name = concat!(
        "services::agent_local::shell_sandbox::tool_roots::tests::",
        "saturated_root_limit_is_silent_and_persists_a_bounded_diagnostic"
    );
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_PATH, &diagnostic)
        .output()
        .expect("child test");

    assert!(output.status.success());
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
    let bytes = std::fs::read(diagnostic).expect("diagnostic");
    assert!(bytes.len() < 256);
    let text = String::from_utf8(bytes).expect("utf8");
    assert!(text.contains("root_read_limit"));
    assert!(!text.contains(&temp.path().to_string_lossy().to_string()));
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, "#!/bin/sh\n").expect("executable");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .expect("permissions");
}

#[cfg(windows)]
fn make_executable(path: &Path) {
    std::fs::write(path.with_extension("exe"), "executable").expect("executable");
}
