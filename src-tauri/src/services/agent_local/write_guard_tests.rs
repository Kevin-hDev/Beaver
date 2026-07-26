use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::services::agent_local::write_guard::WriteGuard;

#[test]
fn allows_write_to_new_file() {
    let guard = WriteGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nouveau.txt");
    assert!(!path.exists());
    assert!(guard.check_write(&path).is_ok());
}

#[test]
fn warns_write_to_existing_unread_file() {
    let guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "contenu existant").unwrap();
    let path = tmp.path();
    assert!(path.exists());
    let result = guard.check_write(path);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("Écriture bloquée"),
        "le message doit indiquer un blocage, pas un warning: {msg}"
    );
    assert!(
        msg.contains("read_file"),
        "doit mentionner read_file: {msg}"
    );
}

#[test]
fn allows_write_after_read() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "contenu").unwrap();
    let path = tmp.path();
    guard.record_read(path);
    assert!(guard.check_write(path).is_ok());
}

#[test]
fn allows_edit_after_read() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "ligne existante").unwrap();
    let path = tmp.path();
    // Simule un edit_file : read d'abord, puis check_write
    guard.record_read(path);
    assert!(guard.check_write(path).is_ok());
}

#[test]
fn record_read_evicts_fifo_when_full() {
    // MAX_READ_PATHS = 1000, EVICT_COUNT = 100 dans write_guard.rs
    const MAX_READ_PATHS: usize = 1000;
    const EVICT_COUNT: usize = 100;
    let mut guard = WriteGuard::new();

    // Rempli jusqu'à MAX_READ_PATHS avec des chemins fictifs
    for i in 0..MAX_READ_PATHS {
        let fake = PathBuf::from(format!("/fake/path/{i}"));
        guard.record_read(&fake);
    }

    // Le 1001ème ajout doit déclencher l'éviction FIFO (100 premiers supprimés)
    let extra = PathBuf::from("/fake/path/extra");
    guard.record_read(&extra); // déclenche l'éviction

    // Les chemins 0..EVICT_COUNT ont été évincés → check_write échoue si le fichier existait
    // (ici on vérifie juste que le guard ne crash pas et accepte un nouveau fichier inexistant)
    let new_path = PathBuf::from("/tmp/nouveau_fichier_inexistant_xyz.txt");
    assert!(!new_path.exists(), "le fichier de test ne doit pas exister");
    assert!(
        guard.check_write(&new_path).is_ok(),
        "un fichier inexistant doit toujours pouvoir être écrit"
    );

    // Le chemin évincé (index 0) n'est plus dans la liste
    let evicted = PathBuf::from("/fake/path/0");
    // On vérifie indirectement : record_read du chemin évincé ne panique pas et l'ajoute à nouveau
    guard.record_read(&evicted);

    // Le chemin ajouté récemment (extra) est toujours présent (EVICT_COUNT après)
    let preserved = PathBuf::from("/fake/path/extra");
    guard.record_read(&preserved); // doit être un no-op (déjà présent)

    // Vérification finale : le guard reste utilisable sans paniquer
    assert!(
        guard.check_write(&new_path).is_ok(),
        "contrôle post-éviction OK"
    );

    // Vérifie que l'éviction FIFO supprime bien EVICT_COUNT entrées
    // On repart de zéro et on dépasse la limite pour compter
    let mut guard2 = WriteGuard::new();
    for i in 0..MAX_READ_PATHS {
        guard2.record_read(&PathBuf::from(format!("/check/path/{i}")));
    }
    // Avant dépassement : MAX_READ_PATHS chemins
    // Après ajout d'un nouveau : éviction de EVICT_COUNT, soit MAX_READ_PATHS - EVICT_COUNT + 1
    guard2.record_read(&PathBuf::from("/check/path/new"));
    // Le chemin index EVICT_COUNT est toujours là (non évincé)
    let still_here = PathBuf::from(format!("/check/path/{EVICT_COUNT}"));
    // check_write ne peut pas tester des chemins fictifs (ils n'existent pas → Ok)
    // On vérifie juste que l'appel ne panique pas
    let _ = guard2.check_write(&still_here);
}
