//! Cas négatifs des validateurs de chemins avec des racines explicites.
//! Ces tests sont indépendants de la configuration réelle de la machine.

use crate::services::agent_local::security::{
    validate_read_path, validate_read_path_in_roots, validate_write_path,
    validate_write_path_in_roots,
};

fn canonical(path: &std::path::Path) -> std::path::PathBuf {
    dunce::canonicalize(path).expect("canonical test directory")
}

fn system_path() -> std::path::PathBuf {
    #[cfg(windows)]
    return std::path::PathBuf::from(r"C:\Windows\cl-go-deny-test");
    #[cfg(not(windows))]
    return std::path::PathBuf::from("/etc/cl-go-deny-test");
}

// --- validate_write_path : cas négatifs -------------------------------------

#[test]
fn write_rejects_path_outside_allowed_zones() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let outside = tempfile::tempdir().expect("outside root");
    let target = outside.path().join("existing.txt");
    std::fs::write(&target, b"").expect("outside test file");

    let result = validate_write_path_in_roots(&target, &[canonical(allowed.path())]);

    assert!(
        result.is_err(),
        "l'écriture d'un fichier existant hors des racines doit être rejetée"
    );
}

#[test]
fn write_rejects_new_file_in_unrelated_directory() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let outside = tempfile::tempdir().expect("outside root");
    let target = outside.path().join("new.txt");

    let result = validate_write_path_in_roots(&target, &[canonical(allowed.path())]);

    assert!(
        result.is_err(),
        "l'écriture d'un nouveau fichier hors des racines doit être rejetée"
    );
}

#[test]
fn write_rejects_dotdot_escape() {
    let base = tempfile::tempdir().expect("test base");
    let allowed = base.path().join("allowed");
    let outside = base.path().join("outside");
    std::fs::create_dir_all(&allowed).expect("allowed root");
    std::fs::create_dir_all(&outside).expect("outside root");
    let escape = allowed.join("../outside/escaped.txt");

    let result = validate_write_path_in_roots(&escape, &[canonical(&allowed)]);

    assert!(
        result.is_err(),
        "un chemin avec .. qui sort de la racine doit être rejeté"
    );
}

#[test]
fn public_write_validator_rejects_path_outside_restrictive_policy() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let roots = vec![canonical(allowed.path())];

    super::super::directory_policy::test_support::with_roots(roots, || {
        assert!(validate_write_path(&system_path(), allowed.path()).is_err());
    });
}

// --- validate_read_path : cas négatifs --------------------------------------

#[test]
fn read_rejects_outside_working_dir_and_roots() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let outside = tempfile::tempdir().expect("outside root");
    let target = outside.path().join("outside.txt");
    std::fs::write(&target, b"").expect("outside test file");

    let result = validate_read_path_in_roots(
        &target,
        allowed.path(),
        &[canonical(allowed.path())],
    );

    assert!(
        result.is_err(),
        "la lecture hors des racines doit être rejetée"
    );
}

#[test]
fn public_read_validator_rejects_path_outside_restrictive_policy() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let roots = vec![canonical(allowed.path())];

    super::super::directory_policy::test_support::with_roots(roots, || {
        assert!(validate_read_path(&system_path(), allowed.path()).is_err());
    });
}

#[test]
fn read_allows_file_in_explicit_root() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let target = allowed.path().join("inside.txt");
    std::fs::write(&target, b"").expect("inside test file");

    let result = validate_read_path_in_roots(
        &target,
        allowed.path(),
        &[canonical(allowed.path())],
    );

    assert!(
        result.is_ok(),
        "la lecture dans une racine explicite doit être autorisée"
    );
}
