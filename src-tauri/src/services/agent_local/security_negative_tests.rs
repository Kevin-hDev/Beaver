//! Cas NÉGATIFS de validate_read_path / validate_write_path : un chemin en
//! dehors des zones autorisées doit être REJETÉ.
//!
//! IMPORTANT : les zones autorisées incluent `config.allowed_paths`. Si cette
//! config contient "/" (tout autorisé, mode développement permissif), les
//! tests négatifs n'ont pas de sens sur cette machine — on les SAUTE avec un
//! message explicite plutôt que de fail.

use crate::services::agent_local::security::{validate_read_path, validate_write_path};
use crate::services::config::read_config;

/// True si la config autorise tout (allowed_paths contient "/" ou vide dans un
/// contexte permissif). Dans ce cas, les tests négatifs sont skippés.
fn config_is_permissive() -> bool {
    match read_config() {
        Ok(c) => c.advanced.allowed_paths.iter().any(|p| p == "/"),
        Err(_) => false,
    }
}

/// Crée un fichier vide hors des zones par défaut (home directement, pas un
/// sous-dossier de data_dir/temp).
fn file_in_home(file_name: &str) -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    let target = home.join(file_name);
    std::fs::write(&target, b"").ok()?;
    Some(target)
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

// --- validate_write_path : cas négatifs -------------------------------------

#[test]
fn write_rejects_path_outside_allowed_zones() {
    if config_is_permissive() {
        eprintln!("[skip] config allowed_paths contient '/' — test négatif sans objet");
        return;
    }
    let Some(target) = file_in_home(".cl-go-deny-write-test") else {
        return;
    };
    let result = validate_write_path(&target);
    cleanup(&target);
    assert!(
        result.is_err(),
        "l'écriture hors des zones autorisées doit être rejetée (path traversal)"
    );
}

#[test]
fn write_rejects_system_directory() {
    if config_is_permissive() {
        eprintln!("[skip] config permissive — test négatif sans objet");
        return;
    }
    // /usr/local : hors data_dir et temp_dir sur la plupart des OS.
    let target = std::path::PathBuf::from("/usr/local/.cl-go-deny-write-test");
    let result = validate_write_path(&target);
    assert!(
        result.is_err(),
        "l'écriture dans /usr/local doit être rejetée hors zones autorisées"
    );
}

#[test]
fn write_rejects_dotdot_escape() {
    if config_is_permissive() {
        eprintln!("[skip] config permissive — test négatif sans objet");
        return;
    }
    let tmp = std::env::temp_dir();
    // ../ depuis temp pour sortir de la zone temp.
    let escape = tmp.join("../../../.cl-go-dotdot-escape-test");
    let result = validate_write_path(&escape);
    cleanup(&std::path::Path::new("/").join(".cl-go-dotdot-escape-test"));
    assert!(
        result.is_err(),
        "un chemin avec ../ qui s'échappe des zones autorisées doit être rejeté"
    );
}

// --- validate_read_path : cas négatifs --------------------------------------

#[test]
fn read_rejects_outside_working_dir_and_roots() {
    if config_is_permissive() {
        eprintln!("[skip] config permissive — test négatif sans objet");
        return;
    }
    let Some(target) = file_in_home(".cl-go-deny-read-test") else {
        return;
    };
    let working = std::env::temp_dir();
    let result = validate_read_path(&target, &working);
    cleanup(&target);
    assert!(
        result.is_err(),
        "la lecture hors working_dir et des roots autorisés doit être rejetée"
    );
}

#[test]
fn read_allows_file_in_working_dir() {
    // Contrôle positif : un fichier DANS working_dir doit toujours passer,
    // indépendamment de la config (working_dir est autorisé par défaut).
    let working = std::env::temp_dir();
    let target = working.join(".cl-go-allow-read-test");
    std::fs::write(&target, b"").unwrap();
    let result = validate_read_path(&target, &working);
    cleanup(&target);
    assert!(
        result.is_ok(),
        "la lecture dans working_dir doit être autorisée"
    );
}
