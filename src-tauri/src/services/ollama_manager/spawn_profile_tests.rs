use super::error::OllamaErrorCode;
#[cfg(unix)]
use super::path_identity::PathIdentityResolver;
use super::path_identity::VerifiedDirectoryLocation;
#[cfg(unix)]
use super::path_identity_resolver::NativePathIdentityResolver;
use super::spawn_profile::OllamaSpawnProfile;
use super::spawn_profile_test_support::{
    directory, existing_location, paths, resolve, FakeResolver, ROOT,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn resolves_absolute_models_path_and_keeps_the_executable_profile() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = resolve(
        &resolver,
        &[
            ("HOME", "/fake/home"),
            ("OLLAMA_MODELS", "/fake/cwd/models"),
        ],
    )
    .expect("profile");
    assert_eq!(
        profile.models_directory().path(),
        Path::new("/fake/cwd/models")
    );
    assert_eq!(profile.working_directory().path(), Path::new("/fake/cwd"));
    assert!(profile.executable().path().is_absolute());
    assert_eq!(profile.environment().get("OLLAMA_NO_CLOUD"), Some("1"));
}

#[test]
fn resolves_relative_models_from_the_profile_cwd_not_process_cwd() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = resolve(
        &resolver,
        &[("HOME", "/fake/home"), ("OLLAMA_MODELS", "models")],
    )
    .expect("profile");
    assert_eq!(
        profile.models_directory().path(),
        Path::new("/fake/cwd/models")
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
    assert_eq!(
        profile.models_directory().path(),
        Path::new("/fake/data/.ollama/models")
    );
    assert_eq!(
        profile.environment().get("OLLAMA_MODELS"),
        Some("/fake/data/.ollama/models")
    );
}

#[test]
fn rejects_models_equal_to_or_overlapping_transaction_locations_in_both_directions() {
    let base = paths();
    let mut resolver = FakeResolver::with_paths(&base);
    let mut locations = (*resolver.locations).clone();
    locations.insert(
        base.active.clone(),
        existing_location("/fake/data/ollama-bundle", 3),
    );
    resolver.locations = Arc::new(locations);
    assert_eq!(
        resolve(&resolver, &[("OLLAMA_MODELS", "/fake/data/ollama-bundle")]),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );

    let model = PathBuf::from("/fake/data/ollama-bundle/models");
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
            Path::new("/fake/cwd"),
            &resolver
        ),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[test]
fn identity_aliases_are_rejected_even_when_lexemes_differ() {
    let base = paths();
    let mut resolver = FakeResolver::with_paths(&base);
    let model = PathBuf::from("/fake/data/alias-models");
    let mut locations = (*resolver.locations).clone();
    locations.insert(
        model.clone(),
        existing_location("/fake/data/alias-models", 7),
    );
    locations.insert(
        base.active.clone(),
        existing_location("/fake/data/ollama-bundle", 7),
    );
    resolver.locations = Arc::new(locations);
    assert_eq!(
        resolve(
            &resolver,
            &[("OLLAMA_MODELS", model.to_str().expect("model"))]
        ),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}

#[test]
fn native_inspection_error_fails_closed_without_mutation() {
    let resolver =
        FakeResolver::with_paths(&paths()).fail_with(OllamaErrorCode::OllamaStorageUnavailable);
    assert_eq!(
        resolve(&resolver, &[("OLLAMA_MODELS", "/fake/cwd/models")]),
        Err(OllamaErrorCode::OllamaStorageUnavailable)
    );
    assert_eq!(resolver.mutation_count(), 0);
}

#[test]
fn probe_profile_has_an_isolated_models_directory() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = OllamaSpawnProfile::resolve_probe(
        &paths(),
        super::spawn_profile_test_support::env(&[
            ("HOME", "/fake/home"),
            ("OLLAMA_MODELS", "/fake/cwd/models"),
        ]),
        Path::new("/fake/cwd"),
        &resolver,
    )
    .expect("probe profile");
    assert_eq!(profile.models_directory().path(), paths().probe_models);
    assert_ne!(
        profile.models_directory().path(),
        Path::new("/fake/cwd/models")
    );
}

#[cfg(unix)]
#[test]
fn unix_resolver_keeps_an_absent_location_to_its_existing_parent() {
    let root = tempfile::tempdir().expect("temp root");
    let paths = crate::services::paths::ollama_paths(root.path());
    let cwd = root.path().join("cwd");
    let home = root.path().join("home");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(home.join(".ollama")).expect("home");
    let resolver = NativePathIdentityResolver;
    let model = root.path().join("models-absent");
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
fn unix_resolver_accepts_unicode_but_rejects_an_internal_nul_before_fs() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().expect("temp root");
    let cwd = root.path().join("cwd");
    std::fs::create_dir_all(&cwd).expect("cwd");
    let paths = crate::services::paths::ollama_paths(root.path());
    let resolver = NativePathIdentityResolver;
    let unicode = root.path().join("模型");
    let profile = OllamaSpawnProfile::resolve(
        &paths,
        [("OLLAMA_MODELS".into(), unicode.as_os_str().into())],
        &cwd,
        &resolver,
    )
    .expect("unicode profile");
    let expected_unicode = dunce::canonicalize(root.path())
        .expect("canonical root")
        .join("模型");
    assert_eq!(profile.models_directory().path(), expected_unicode);
    let nul = std::ffi::OsString::from_vec(b"/tmp/models\0hidden".to_vec());
    assert_eq!(
        OllamaSpawnProfile::resolve(&paths, [("OLLAMA_MODELS".into(), nul)], &cwd, &resolver),
        Err(OllamaErrorCode::OllamaInternal)
    );
}
