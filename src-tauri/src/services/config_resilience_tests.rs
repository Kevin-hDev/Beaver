//! Tests de résilience de la persistence config (FS-tmp).
//!
//! La lecture tolérante est CRITIQUE : un config corrompu ou partiel ne doit
//! JAMAIS crasher l'app. On valide :
//!   - fichier absent → config par défaut
//!   - JSON corrompu → config par défaut + sentinelle .config-corrupted
//!   - wakeup invalide mélangé à des valides → valides conservés, invalides
//!     droppés (résilience section par section)
//!   - écriture atomique (pas de .tmp résiduel)

use crate::models::{ClgoConfig, ScheduledWakeup, WakeupSchedule};
use crate::services::config::{read_config_from_path, write_config_to_path};
use serde_json::json;

/// Wakeup valide minimal (Daily).
fn valid_wakeup(id: &str) -> ScheduledWakeup {
    ScheduledWakeup {
        id: id.to_string(),
        name: "test".to_string(),
        model: "gpt".to_string(),
        provider: "openai".to_string(),
        prompt: "hello".to_string(),
        schedule: WakeupSchedule::Daily {
            time: "08:00".to_string(),
        },
        description: String::new(),
        active: true,
        paused_by_global: false,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

// --- Lecture : fichier absent -----------------------------------------------

#[test]
fn missing_file_returns_default_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");

    let config = read_config_from_path(&path, dir.path()).expect("read");

    // Une config absente → config vide (pas de wakeup, heartbeat default).
    // On vérifie des champs représentatifs (ClgoConfig n'implémente pas PartialEq).
    assert!(config.scheduled_wakeups.is_empty());
    assert!(!config.heartbeat.global_paused);
}

// --- Lecture : JSON corrompu → default + sentinelle -------------------------

#[test]
fn corrupted_json_returns_default_and_writes_sentinel() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, b"{ this is not valid json }").unwrap();

    let config = read_config_from_path(&path, dir.path()).expect("read");

    // Doit retourner une config par défaut, pas crasher.
    assert!(config.scheduled_wakeups.is_empty());
    assert!(!config.heartbeat.global_paused);

    // Une sentinelle .config-corrupted doit être créée (pour audit/debug).
    let sentinel = dir.path().join(".config-corrupted");
    assert!(
        sentinel.exists(),
        "la sentinelle de corruption doit être écrite pour audit"
    );
}

#[test]
fn corrupted_json_does_not_reveal_internal_paths_in_default() {
    // La config par défaut retournée ne doit pas contenir de traces du
    // fichier source (pas de fuite de chemin dans le retour).
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, b"garbage").unwrap();

    let config = read_config_from_path(&path, dir.path()).expect("read");
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(
        !serialized.contains(&path.to_string_lossy().to_string()),
        "la config par défaut ne doit pas fuiter le chemin du fichier"
    );
}

// --- Lecture : tolérance section par section --------------------------------

#[test]
fn partial_config_keeps_valid_sections() {
    // Un config avec un heartbeat valide mais un advanced invalide doit
    // garder le heartbeat et reset l'advanced.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let content = json!({
        "heartbeat": { "global_paused": true },
        "advanced": "this should be an object not a string"
    });
    std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

    let config = read_config_from_path(&path, dir.path()).expect("read");

    assert!(
        config.heartbeat.global_paused,
        "heartbeat valide doit être conservé"
    );
    // advanced invalide → reset à default (compression_threshold par défaut).
    // On compare un champ représentatif plutôt que la struct entière (pas de
    // derive PartialEq sur AdvancedSettings).
    assert_eq!(
        config.advanced.compression_threshold,
        ClgoConfig::default().advanced.compression_threshold,
        "advanced invalide doit être reset à default"
    );
}

#[test]
fn invalid_wakeup_dropped_valid_wakeup_kept() {
    // Mélange d'un wakeup valide et d'un invalide : seul le valide survit.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let valid = serde_json::to_value(valid_wakeup("valid-1")).unwrap();
    let invalid = json!({ "id": "bad", "missing_required_fields": true });
    let content = json!({
        "scheduled_wakeups": [valid, invalid]
    });
    std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

    let config = read_config_from_path(&path, dir.path()).expect("read");

    assert_eq!(
        config.scheduled_wakeups.len(),
        1,
        "le wakeup invalide doit être droppé, le valide conservé"
    );
    assert_eq!(config.scheduled_wakeups[0].id, "valid-1");
}

#[test]
fn all_invalid_wakeups_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let content = json!({
        "scheduled_wakeups": [
            { "bad": 1 },
            { "also_bad": 2 }
        ]
    });
    std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

    let config = read_config_from_path(&path, dir.path()).expect("read");

    assert!(
        config.scheduled_wakeups.is_empty(),
        "tous les wakeups invalides doivent être droppés"
    );
}

// --- Écriture atomique ------------------------------------------------------

#[test]
fn write_config_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut config = ClgoConfig::default();
    config.heartbeat.global_paused = true;
    config.scheduled_wakeups.push(valid_wakeup("w1"));

    write_config_to_path(&path, &config).expect("write");

    let loaded = read_config_from_path(&path, dir.path()).expect("read");
    assert!(loaded.heartbeat.global_paused);
    assert_eq!(loaded.scheduled_wakeups.len(), 1);
    assert_eq!(loaded.scheduled_wakeups[0].id, "w1");
}

#[test]
fn write_config_leaves_no_tmp_residue() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");

    write_config_to_path(&path, &ClgoConfig::default()).expect("write");

    let tmp = dir.path().join("config.json.tmp");
    assert!(
        !tmp.exists(),
        "aucun .tmp résiduel ne doit subsister après écriture atomique"
    );
    assert!(path.exists());
}

#[test]
fn write_config_creates_parent_dir() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("deep").join("path");
    let path = nested.join("config.json");

    write_config_to_path(&path, &ClgoConfig::default()).expect("write");

    assert!(path.exists());
}
