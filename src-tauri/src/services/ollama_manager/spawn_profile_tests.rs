use super::error::OllamaErrorCode;
#[cfg(unix)]
use super::path_identity::PathIdentityResolver;
use super::path_identity::VerifiedDirectoryLocation;
#[cfg(unix)]
use super::path_identity_resolver::NativePathIdentityResolver;
use super::spawn_profile::OllamaSpawnProfile;
use super::spawn_profile_test_support::{
    directory, existing_location, paths, resolve, FakeResolver, CWD, HOME, ROOT,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(unix)]
fn create_active_executable(paths: &crate::services::paths::OllamaPaths, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let executable = paths.active.join("bin").join("ollama");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("executable directory");
    std::fs::write(&executable, b"ollama-test").expect("executable file");
    let mut permissions = std::fs::metadata(&executable)
        .expect("executable metadata")
        .permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(executable, permissions).expect("executable permissions");
}

#[test]
fn resolves_absolute_models_path_and_keeps_the_executable_profile() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = resolve(
        &resolver,
        &[
            ("HOME", HOME),
            (
                "OLLAMA_MODELS",
                &PathBuf::from(CWD).join("models").to_string_lossy(),
            ),
        ],
    )
    .expect("profile");
    assert_eq!(
        profile.models_directory().path(),
        PathBuf::from(CWD).join("models")
    );
    assert_eq!(profile.working_directory().path(), Path::new(CWD));
    assert!(profile.executable().path().is_absolute());
    assert_eq!(profile.environment().get("OLLAMA_NO_CLOUD"), Some("1"));
}

#[test]
fn resolves_relative_models_from_the_profile_cwd_not_process_cwd() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile =
        resolve(&resolver, &[("HOME", HOME), ("OLLAMA_MODELS", "models")]).expect("profile");
    assert_eq!(
        profile.models_directory().path(),
        PathBuf::from(CWD).join("models")
    );
}

#[test]
fn rejects_empty_and_parent_models_values_before_identity_mutation() {
    for value in ["", "../models", "/fake/cwd/../models", "."] {
        let resolver = FakeResolver::with_paths(&paths());
        assert_eq!(
            resolve(
                &resolver,
                &[("HOME", "/fake/home"), ("OLLAMA_MODELS", value)]
            ),
            Err(OllamaErrorCode::OllamaModelStoreConflict),
            "{value}"
        );
        assert_eq!(resolver.mutation_count(), 0);
    }
}

#[test]
fn absent_models_uses_the_same_inherited_home_authority() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = resolve(&resolver, &[("HOME", ROOT), ("PATH", "/bin")]).expect("profile");
    let expected_models = PathBuf::from(ROOT).join(".ollama").join("models");
    assert_eq!(profile.models_directory().path(), expected_models);
    assert_eq!(
        profile.environment().get("OLLAMA_MODELS"),
        expected_models.to_str()
    );
}

#[test]
fn rejects_models_equal_to_or_overlapping_transaction_locations_in_both_directions() {
    let base = paths();
    let mut resolver = FakeResolver::with_paths(&base);
    let mut locations = (*resolver.locations).clone();
    let active = base.active.to_string_lossy().into_owned();
    locations.insert(base.active.clone(), existing_location(&active, 3));
    resolver.locations = Arc::new(locations);
    assert_eq!(
        resolve(&resolver, &[("OLLAMA_MODELS", &active)]),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );

    let model = base.active.join("models");
    let mut locations = (*resolver.locations).clone();
    locations.insert(
        model.clone(),
        VerifiedDirectoryLocation::existing(directory(model.to_str().expect("model"), 4)),
    );
    resolver.locations = Arc::new(locations);
    assert_eq!(
        OllamaSpawnProfile::resolve(
            &base,
            super::spawn_profile_test_support::env(&[(
                "OLLAMA_MODELS",
                model.to_str().expect("model")
            )]),
            Path::new(CWD),
            &resolver
        ),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[test]
fn identity_aliases_are_rejected_even_when_lexemes_differ() {
    let base = paths();
    let mut resolver = FakeResolver::with_paths(&base);
    let model = PathBuf::from(ROOT).join("alias-models");
    let model_text = model.to_string_lossy().into_owned();
    let active = base.active.to_string_lossy().into_owned();
    let mut locations = (*resolver.locations).clone();
    locations.insert(model.clone(), existing_location(&model_text, 7));
    locations.insert(base.active.clone(), existing_location(&active, 7));
    resolver.locations = Arc::new(locations);
    assert_eq!(
        resolve(&resolver, &[("OLLAMA_MODELS", &model_text)]),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[test]
fn native_inspection_error_fails_closed_without_mutation() {
    let resolver =
        FakeResolver::with_paths(&paths()).fail_with(OllamaErrorCode::OllamaStorageUnavailable);
    assert_eq!(
        resolve(
            &resolver,
            &[(
                "OLLAMA_MODELS",
                &PathBuf::from(CWD).join("models").to_string_lossy()
            )]
        ),
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
    assert_eq!(resolver.mutation_count(), 0);
}

#[test]
fn probe_profile_has_an_isolated_models_directory() {
    let resolver = FakeResolver::with_paths(&paths());
    let models = PathBuf::from(CWD).join("models");
    let profile = OllamaSpawnProfile::resolve_probe(
        &paths(),
        super::spawn_profile_test_support::env(&[
            ("HOME", HOME),
            ("OLLAMA_MODELS", models.to_str().expect("models")),
        ]),
        Path::new(CWD),
        &resolver,
    )
    .expect("probe profile");
    assert_eq!(profile.models_directory().path(), paths().probe_models);
    assert_ne!(profile.models_directory().path(), models);
}

#[cfg(unix)]
#[test]
fn unix_resolver_keeps_an_absent_location_to_its_existing_parent() {
    let root = tempfile::tempdir().expect("temp root");
    let root_path = dunce::canonicalize(root.path()).expect("canonical root");
    let paths = crate::services::paths::ollama_paths(&root_path);
    let cwd = root_path.join("cwd");
    let home = root_path.join("home");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(home.join(".ollama")).expect("home");
    create_active_executable(&paths, 0o755);
    let resolver = NativePathIdentityResolver;
    let model = root_path.join("models-absent");
    let profile = OllamaSpawnProfile::resolve(
        &paths,
        [
            ("HOME".into(), home.as_os_str().into()),
            ("OLLAMA_MODELS".into(), model.as_os_str().into()),
        ],
        &cwd,
        &resolver,
    )
    .expect("profile");
    let expected_model = dunce::canonicalize(root.path())
        .expect("canonical root")
        .join("models-absent");
    assert_eq!(profile.models_directory().path(), expected_model);
    assert!(profile.working_directory().has_stable_handle());
    assert!(profile.models_directory().has_stable_handle());
    assert!(profile.executable().has_stable_handle());
    assert!(!model.exists());
}

#[cfg(unix)]
#[test]
fn unix_resolver_rejects_symlink_alias_before_profile_acceptance() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().expect("temp root");
    let real = root.path().join("real");
    let link = root.path().join("link");
    std::fs::create_dir_all(&real).expect("real");
    symlink(&real, &link).expect("symlink");
    let resolver = NativePathIdentityResolver;
    assert_eq!(
        resolver.verified_location(&link),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_rejects_a_symlink_ancestor_for_an_absent_location() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(real.join("nested")).expect("real");
    symlink(&real, &link).expect("symlink");
    let resolver = NativePathIdentityResolver;
    assert_eq!(
        resolver.verified_location(&link.join("nested").join("models")),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_allows_only_a_single_absent_final_component() {
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let resolver = NativePathIdentityResolver;
    assert!(resolver.verified_location(&root.join("new-models")).is_ok());
    assert_eq!(
        resolver.verified_location(&root.join("missing-parent").join("models")),
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
    assert_eq!(
        resolver.verified_location(&PathBuf::from("/dev/null").join("models")),
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_maps_permission_errors_to_storage_unavailable() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let denied = root.join("denied");
    std::fs::create_dir(&denied).expect("denied directory");
    let mut permissions = std::fs::metadata(&denied)
        .expect("denied metadata")
        .permissions();
    permissions.set_mode(0o0);
    std::fs::set_permissions(&denied, permissions.clone()).expect("deny directory");
    let result = NativePathIdentityResolver.verified_location(&denied.join("models"));
    permissions.set_mode(0o755);
    std::fs::set_permissions(&denied, permissions).expect("restore directory");
    assert_eq!(result, Err(OllamaErrorCode::OllamaStorageUnavailable));
}

#[cfg(unix)]
#[test]
fn unix_resolver_contains_uses_native_ancestors_in_both_directions() {
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let child_path = root.join("child");
    std::fs::create_dir(&child_path).expect("child");
    let resolver = NativePathIdentityResolver;
    let parent = resolver.canonical_directory(&root).expect("parent");
    let child = resolver.canonical_directory(&child_path).expect("child");
    assert!(resolver
        .contains(&parent, &child)
        .expect("parent contains child"));
    assert!(!resolver
        .contains(&child, &parent)
        .expect("child contains parent"));
}

#[cfg(unix)]
#[test]
fn unix_profile_requires_a_regular_executable_file() {
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let paths = crate::services::paths::ollama_paths(&root);
    let cwd = root.join("cwd");
    let home = root.join("home");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(home.join(".ollama")).expect("home");
    let resolver = NativePathIdentityResolver;
    let inherited = [
        ("HOME".into(), home.as_os_str().into()),
        (
            "OLLAMA_MODELS".into(),
            root.join("models").as_os_str().into(),
        ),
    ];
    assert_eq!(
        OllamaSpawnProfile::resolve(&paths, inherited, &cwd, &resolver),
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_accepts_unicode_but_rejects_an_internal_nul_before_fs() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().expect("temp root");
    let root_path = dunce::canonicalize(root.path()).expect("canonical root");
    let cwd = root_path.join("cwd");
    std::fs::create_dir_all(&cwd).expect("cwd");
    let paths = crate::services::paths::ollama_paths(&root_path);
    let resolver = NativePathIdentityResolver;
    create_active_executable(&paths, 0o755);
    let unicode = root_path.join("模型");
    let profile = OllamaSpawnProfile::resolve(
        &paths,
        [("OLLAMA_MODELS".into(), unicode.as_os_str().into())],
        &cwd,
        &resolver,
    )
    .expect("unicode profile");
    let expected_unicode = root_path.join("模型");
    assert_eq!(profile.models_directory().path(), expected_unicode);
    let nul = std::ffi::OsString::from_vec(b"/tmp/models\0hidden".to_vec());
    assert_eq!(
        OllamaSpawnProfile::resolve(&paths, [("OLLAMA_MODELS".into(), nul)], &cwd, &resolver),
        Err(OllamaErrorCode::OllamaInternal)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_rejects_a_non_executable_active_file() {
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let paths = crate::services::paths::ollama_paths(&root);
    let cwd = root.join("cwd");
    let home = root.join("home");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(home.join(".ollama")).expect("home");
    create_active_executable(&paths, 0o644);
    let resolver = NativePathIdentityResolver;
    assert_eq!(
        OllamaSpawnProfile::resolve(
            &paths,
            [
                ("HOME".into(), home.as_os_str().into()),
                (
                    "OLLAMA_MODELS".into(),
                    root.join("models").as_os_str().into()
                ),
            ],
            &cwd,
            &resolver,
        ),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_rejects_a_symlink_active_file() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let paths = crate::services::paths::ollama_paths(&root);
    let cwd = root.join("cwd");
    let home = root.join("home");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(home.join(".ollama")).expect("home");
    let executable = paths.active.join("bin").join("ollama");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("executable directory");
    let target = root.join("real-ollama");
    std::fs::write(&target, b"ollama-test").expect("target");
    let mut permissions = std::fs::metadata(&target)
        .expect("target metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&target, permissions).expect("target permissions");
    symlink(&target, &executable).expect("executable symlink");
    let resolver = NativePathIdentityResolver;
    assert_eq!(
        OllamaSpawnProfile::resolve(
            &paths,
            [
                ("HOME".into(), home.as_os_str().into()),
                (
                    "OLLAMA_MODELS".into(),
                    root.join("models").as_os_str().into()
                ),
            ],
            &cwd,
            &resolver,
        ),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[cfg(unix)]
#[test]
fn unix_executable_identity_survives_path_replacement() {
    let root = tempfile::tempdir().expect("temp root");
    let root = dunce::canonicalize(root.path()).expect("canonical root");
    let paths = crate::services::paths::ollama_paths(&root);
    create_active_executable(&paths, 0o755);
    let executable = paths.active.join("bin").join("ollama");
    let resolver = NativePathIdentityResolver;
    let first = resolver
        .canonical_executable(&executable)
        .expect("first executable");
    std::fs::rename(&executable, root.join("old-ollama")).expect("replace old executable");
    create_active_executable(&paths, 0o755);
    let second = resolver
        .canonical_executable(&executable)
        .expect("second executable");
    assert_ne!(first.identity(), second.identity());
    assert!(first.has_stable_handle());
    assert!(second.has_stable_handle());
}
