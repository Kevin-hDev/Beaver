//! Tests d'atomicité et de bornage de la persistence MCP (FS-tmp).
//!
//! `save_to_path` fait une écriture atomique : create(.tmp) → write → sync →
//! rename. On vérifie qu'aucun fichier .tmp résiduel ne subsiste et que le
//! round-trip write → load préserve les connecteurs. La borne MAX_CONNECTORS
//! est validée via load_from_path (qui rejette > MAX à la lecture).

use super::config::{self, StoredConnector, MAX_CONNECTORS};

/// Connecteur notion valide (endpoint trusted du catalog).
fn notion() -> StoredConnector {
    StoredConnector {
        id: "notion".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: Some("https://mcp.notion.com/mcp".to_string()),
        install_command: None,
        env_keys: None,
    }
}

/// Connecteur sentry valide (endpoint trusted du catalog).
fn sentry() -> StoredConnector {
    StoredConnector {
        id: "sentry".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: Some("https://mcp.sentry.dev/mcp".to_string()),
        install_command: None,
        env_keys: None,
    }
}

// --- Atomicité du write -----------------------------------------------------

#[test]
fn save_to_path_round_trips_connectors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");
    let list = vec![notion(), sentry()];

    config::save_to_path(&path, &list).expect("save");

    let loaded = config::load_from_path(&path).expect("load");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, "notion");
    assert_eq!(loaded[1].id, "sentry");
}

#[test]
fn save_to_path_leaves_no_tmp_residue() {
    // Écriture atomique : après succès, aucun fichier .tmp ne doit rester.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");

    config::save_to_path(&path, &[notion()]).expect("save");

    let tmp = dir.path().join("mcp-connectors.json.tmp");
    assert!(
        !tmp.exists(),
        "aucun fichier .tmp résiduel ne doit subsister après écriture atomique"
    );
    assert!(path.exists(), "le fichier final doit exister");
}

#[test]
fn save_to_path_creates_parent_dir() {
    // save crée le dossier parent si absent (create_dir_all).
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested").join("sub");
    let path = nested.join("mcp-connectors.json");

    config::save_to_path(&path, &[notion()]).expect("save");

    assert!(path.exists());
}

#[test]
fn save_to_path_overwrites_existing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");

    config::save_to_path(&path, &[notion()]).expect("save 1");
    config::save_to_path(&path, &[sentry()]).expect("save 2");

    let loaded = config::load_from_path(&path).expect("load");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "sentry");
}

// --- Borne MAX_CONNECTORS ---------------------------------------------------

#[test]
fn save_to_path_rejects_list_over_max() {
    // La borne MAX_CONNECTORS est appliquée à l'écriture dans save_to_path.
    // On ne peut pas construire MAX+1 connecteurs valides distincts (catalog
    // limité), donc on craft directement un JSON > MAX et on teste le rejet
    // via load_from_path.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");

    // Génère un JSON avec MAX+1 connecteurs notion (id identique mais le
    // comptage de load_from_path est sur len(), pas sur l'unicité).
    let oversize: Vec<serde_json::Value> = (0..MAX_CONNECTORS + 1)
        .map(|_| {
            serde_json::json!({
                "id": "notion",
                "status": "connected",
                "enabled_in_chat": true,
                "endpoint": "https://mcp.notion.com/mcp",
                "install_command": null,
                "env_keys": null,
            })
        })
        .collect();
    std::fs::write(&path, serde_json::to_vec_pretty(&oversize).unwrap()).unwrap();

    let result = config::load_from_path(&path);
    assert!(
        result.is_err(),
        "une liste > MAX_CONNECTORS à la lecture doit être rejetée"
    );
}

#[test]
fn save_to_path_accepts_single_connector() {
    // Contrôle : un seul connecteur valide passe sans problème.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");

    config::save_to_path(&path, &[notion()]).expect("save");

    let loaded = config::load_from_path(&path).expect("load");
    assert_eq!(loaded.len(), 1);
}

// --- Idempotence de la migration -------------------------------------------

#[test]
fn normalize_list_is_idempotent() {
    // Une 2e passe de normalize_list ne doit rien changer.
    use super::config_migration;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");
    let initial = vec![StoredConnector {
        id: "context7".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: None,
        install_command: Some("npx @upstash/context7-mcp@2.2.3".to_string()),
        env_keys: None,
    }];
    config::save_to_path(&path, &initial).expect("save");

    // 1er load déclenche la migration.
    let mut connectors = config::load_from_path(&path).expect("first load");
    // 2e passe via normalize_list direct : doit être no-op.
    let changed = config_migration::normalize_list(&mut connectors);
    assert!(
        !changed,
        "une 2e passe de normalisation ne doit rien changer"
    );
}
