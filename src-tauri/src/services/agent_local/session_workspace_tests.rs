use super::*;

#[tokio::test]
async fn creates_private_work_and_outputs_directories() {
    let root = tempfile::tempdir().unwrap();
    let id = uuid::Uuid::new_v4().to_string();

    let workspace = ensure_layout(
        &root.path().join("session-workspaces"),
        None,
        "2026-07-29",
        "Créer une présentation / été",
        &id,
    )
    .await
    .unwrap();

    assert!(workspace.work.is_dir());
    assert!(workspace.outputs.is_dir());
    assert!(workspace
        .work
        .to_string_lossy()
        .contains("créer-une-présentation"));
    assert!(workspace.work.ends_with("work"));
}

#[tokio::test]
async fn the_same_session_reuses_the_same_workspace() {
    let root = tempfile::tempdir().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let base = root.path().join("session-workspaces");

    let first = ensure_layout(&base, None, "2026-07-29", "Analyse", &id)
        .await
        .unwrap();
    let second = ensure_layout(&base, None, "2026-07-29", "Analyse", &id)
        .await
        .unwrap();

    assert_eq!(first.work, second.work);
    assert_eq!(first.outputs, second.outputs);
}

#[cfg(windows)]
#[tokio::test]
async fn reuses_a_workspace_saved_with_a_verbatim_windows_prefix() {
    let root = tempfile::tempdir().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let base = root.path().join("session-workspaces");
    let first = ensure_layout(&base, None, "2026-07-29", "Analyse", &id)
        .await
        .unwrap();
    let verbatim_work = PathBuf::from(format!(r"\\?\{}", first.work.display()));

    let second = ensure_work_path(&base, &verbatim_work, None).await.unwrap();

    assert_eq!(first.work, second.work);
    assert_eq!(first.outputs, second.outputs);
}

#[cfg(windows)]
#[tokio::test]
async fn reuses_an_existing_workspace_beyond_the_legacy_windows_path_limit() {
    let root = tempfile::tempdir().unwrap();
    let mut base = root.path().to_path_buf();
    while base.as_os_str().len() < 205 {
        base.push("profile-segment");
    }
    base.push("session-workspaces");
    assert!(base.as_os_str().len() < 260);
    std::fs::create_dir_all(&base).unwrap();

    let normal_work = base
        .join("2026-08-08")
        .join("analyse-with-a-deliberately-long-name-12345678")
        .join("work");
    assert!(normal_work.as_os_str().len() > 260);
    let verbatim_work = PathBuf::from(format!(r"\\?\{}", normal_work.display()));
    std::fs::create_dir_all(&verbatim_work).unwrap();

    let result = ensure_work_path(&base, &verbatim_work, None).await;
    assert!(verbatim_work.parent().unwrap().join("outputs").is_dir());
    let workspace = result.expect("reuse long workspace");

    assert!(workspace.work.is_dir());
    assert!(workspace.outputs.is_dir());
}

#[cfg(windows)]
#[test]
fn network_workspace_namespaces_resolve_to_the_same_relative_path() {
    let base = PathBuf::from(r"\\server\share\Beaver\session-workspaces");
    let relative = PathBuf::from(r"2026-08-08\analyse-12345678\work");
    let work = PathBuf::from(
        r"\\?\UNC\server\share\Beaver\session-workspaces\2026-08-08\analyse-12345678\work",
    );
    let canonical_base = PathBuf::from(r"\\?\UNC\server\share\Beaver\session-workspaces");
    let canonical_work = canonical_base.join(&relative);

    let resolved = relative_workspace_path_with(&base, &work, |path| {
        if path == base {
            Ok(canonical_base.clone())
        } else if path == work {
            Ok(canonical_work.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "unexpected path",
            ))
        }
    })
    .expect("network workspace relative path");

    assert_eq!(resolved, relative);
}

#[cfg(windows)]
#[test]
fn long_workspace_namespaces_resolve_to_the_same_relative_path() {
    let long_home = format!(r"C:\{}", vec!["profile-segment"; 14].join(r"\"));
    let base = PathBuf::from(long_home).join(r".local\share\cl-go-dash\session-workspaces");
    let relative = PathBuf::from(r"2026-08-08\analyse-12345678\work");
    let canonical_base = PathBuf::from(format!(r"\\?\{}", base.display()));
    let canonical_work = canonical_base.join(&relative);
    assert!(canonical_work.as_os_str().len() > 260);

    let resolved = relative_workspace_path_with(&base, &canonical_work, |path| {
        if path == base {
            Ok(canonical_base.clone())
        } else if path == canonical_work {
            Ok(canonical_work.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "unexpected path",
            ))
        }
    })
    .expect("long workspace relative path");

    assert_eq!(resolved, relative);
}

#[cfg(windows)]
#[test]
fn canonical_fallback_rejects_a_workspace_outside_the_base() {
    let base = PathBuf::from(r"\\server\share\Beaver\session-workspaces");
    let work = PathBuf::from(r"\\?\UNC\server\share\outside\2026-08-08\session\work");

    let result = relative_workspace_path_with(&base, &work, |path| {
        if path == base {
            Ok(PathBuf::from(
                r"\\?\UNC\server\share\Beaver\session-workspaces",
            ))
        } else {
            Ok(PathBuf::from(
                r"\\?\UNC\server\share\outside\2026-08-08\session\work",
            ))
        }
    });

    assert!(result.is_err());
}

#[test]
fn reserved_and_unusable_names_have_a_safe_fallback() {
    assert_eq!(slugify("CON"), "session");
    assert_eq!(slugify("///"), "session");
    assert_eq!(slugify("Hello, World!"), "hello-world");
}

#[test]
fn unicode_labels_remain_readable() {
    assert_eq!(slugify("Créer une présentation"), "créer-une-présentation");
    assert_eq!(
        slugify("プレゼンテーションを作成"),
        "プレゼンテーションを作成"
    );
    assert_eq!(slugify("创建演示文稿"), "创建演示文稿");
}

#[test]
fn a_very_long_label_is_bounded() {
    let label = "x".repeat(100_000);

    assert_eq!(slugify(&label).len(), SLUG_MAX_CHARS);
}

#[tokio::test]
async fn a_symlinked_workspace_component_is_rejected() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let base = root.path().join("session-workspaces");
        let date = base.join("2026-07-29");
        std::fs::create_dir_all(&date).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let name = format!("analyse-{}", session_suffix(&id).unwrap());
        symlink(outside.path(), date.join(name)).unwrap();

        assert!(ensure_layout(&base, None, "2026-07-29", "Analyse", &id)
            .await
            .is_err());
        assert!(!outside.path().join("work").exists());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn a_symlinked_workspace_alias_is_rejected_after_canonical_fallback() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let base = root.path().join("session-workspaces");
    let alias = root.path().join("workspace-alias");
    let work = base
        .join("2026-08-08")
        .join("analyse-12345678")
        .join("work");
    std::fs::create_dir_all(&work).unwrap();
    symlink(&base, &alias).unwrap();
    let aliased_work = alias
        .join("2026-08-08")
        .join("analyse-12345678")
        .join("work");

    assert!(ensure_work_path(&base, &aliased_work, None).await.is_err());
}

#[tokio::test]
async fn custom_outputs_stay_separate_from_work() {
    let root = tempfile::tempdir().unwrap();
    let custom = tempfile::tempdir().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let base = root.path().join("session-workspaces");

    let workspace = ensure_layout(
        &base,
        Some(custom.path()),
        "2026-07-29",
        "Rapport final",
        &id,
    )
    .await
    .unwrap();

    assert!(workspace.work.starts_with(&base));
    assert!(workspace.outputs.starts_with(custom.path()));
    assert!(workspace.outputs.ends_with("outputs"));
}

#[test]
fn managed_access_is_limited_to_one_session_workspace() {
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let base = crate::services::paths::data_dir().join("session-workspaces");
    let session = base.join("2026-08-03").join(unique);
    let work = session.join("work");
    let nested = work.join("src");
    let outputs = session.join("outputs");
    std::fs::create_dir_all(&nested).expect("work");
    std::fs::create_dir_all(&outputs).expect("outputs");

    let roots = access_roots_for(&nested);

    assert!(roots.contains(&dunce::canonicalize(&work).expect("canonical work")));
    assert!(roots.contains(&dunce::canonicalize(&outputs).expect("canonical outputs")));
    assert!(!roots.iter().any(|root| root == &base));
    let _ = std::fs::remove_dir_all(session);
}
