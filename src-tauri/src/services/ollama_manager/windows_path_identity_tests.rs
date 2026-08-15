#![cfg(windows)]

use super::error::OllamaErrorCode;
use super::path_identity::{CanonicalDirectory, NativeDirectoryIdentity, PathIdentityResolver};
use super::path_identity_resolver::NativePathIdentityResolver;
use super::spawn_environment::FrozenEnvironment;
use super::spawn_profile::OllamaSpawnProfile;
use super::spawn_profile_paths::resolve_models_path;
use super::spawn_profile_test_support::{paths, FakeResolver, CWD};
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

#[test]
fn windows_identity_compares_case_aliases_by_native_identity() {
    let resolver = NativePathIdentityResolver;
    let left = CanonicalDirectory::synthetic(
        PathBuf::from(r"C:\Data\Models"),
        Some(NativeDirectoryIdentity::synthetic(7)),
    );
    let right = CanonicalDirectory::synthetic(
        PathBuf::from(r"c:\data\models"),
        Some(NativeDirectoryIdentity::synthetic(7)),
    );
    assert!(resolver
        .same_directory(&left, &right)
        .expect("identity comparison"));
}

#[test]
fn windows_identity_rejects_parent_relationship_for_equal_identity() {
    let resolver = NativePathIdentityResolver;
    let parent = CanonicalDirectory::synthetic(
        PathBuf::from(r"C:\Data"),
        Some(NativeDirectoryIdentity::synthetic(9)),
    );
    let child = CanonicalDirectory::synthetic(
        PathBuf::from(r"C:\Data\Models"),
        Some(NativeDirectoryIdentity::synthetic(9)),
    );
    assert!(!resolver
        .contains(&parent, &child)
        .expect("identity comparison"));
}

#[test]
fn windows_environment_keys_are_deduplicated_without_case() {
    let resolver = FakeResolver::with_paths(&paths());
    let result = OllamaSpawnProfile::resolve(
        &paths(),
        [
            (OsString::from("Path"), OsString::from("one")),
            (OsString::from("pAtH"), OsString::from("two")),
        ],
        Path::new(r"C:\work"),
        &resolver,
    );
    assert_eq!(result, Err(OllamaErrorCode::OllamaInternal));
}

#[test]
fn windows_unicode_and_nul_values_are_checked_in_utf16_units() {
    use std::os::windows::ffi::OsStringExt;
    let nul = OsString::from_wide(&['m' as u16, 0, 'o' as u16]);
    let resolver = FakeResolver::with_paths(&paths());
    let result = OllamaSpawnProfile::resolve(
        &paths(),
        [(OsString::from("OLLAMA_MODELS"), nul)],
        Path::new(r"C:\work"),
        &resolver,
    );
    assert_eq!(result, Err(OllamaErrorCode::OllamaInternal));
}

#[test]
fn windows_host_override_is_rejected_without_case_aliases() {
    let resolver = FakeResolver::with_paths(&paths());
    let models = PathBuf::from(CWD).join("models");
    let result = OllamaSpawnProfile::resolve_with_overrides(
        &paths(),
        [(OsString::from("OLLAMA_MODELS"), models.into_os_string())],
        Path::new(CWD),
        &resolver,
        [(OsString::from("ollama_host"), OsString::from("late"))],
    );
    assert_eq!(result, Err(OllamaErrorCode::OllamaInternal));
}

#[test]
fn windows_drive_relative_models_are_not_resolved_by_a_second_cwd_authority() {
    let cwd = super::spawn_profile_test_support::directory(r"C:\fake\cwd", 2);
    let environment = FrozenEnvironment::from_entries(vec![(
        OsString::from("OLLAMA_MODELS"),
        OsString::from(r"C:models"),
    )]);
    assert_eq!(
        resolve_models_path(environment.value("OLLAMA_MODELS"), &cwd, &environment),
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
}
