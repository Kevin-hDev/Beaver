/* Ordre choisi à la main dans une liste de conversations — celles d'un projet,
   celles qui n'appartiennent à aucun, et celles épinglées en tête de la barre
   latérale.

   Un seul fichier le porte, et les conversations elles-mêmes l'ignorent. Deux
   raisons. L'index des conversations est reconstruit à partir de leurs
   fichiers : un ordre écrit là-bas y serait recopié, et les deux copies
   divergeraient au premier rebuild. Et déplacer une conversation réécrirait
   alors chaque fichier de la liste, qui pèsent leur historique complet. */

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/* Clé de la liste des conversations hors projet. Aucune collision possible
   avec une clé de projet, qui est un UUID. */
pub const ORPHAN_LIST: &str = "orphan";
/* Clé de la liste des conversations épinglées, affichées en tête de la barre
   latérale. Même garantie d'absence de collision. */
pub const PINNED_LIST: &str = "pinned";

const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct SessionOrderFile {
    version: u32,
    lists: HashMap<String, Vec<String>>,
}

impl Default for SessionOrderFile {
    fn default() -> Self {
        Self {
            version: VERSION,
            lists: HashMap::new(),
        }
    }
}

fn order_path() -> PathBuf {
    crate::services::paths::data_dir().join("session-order.json")
}

/* Lecture tolérante : un fichier absent, tronqué ou écrit par une version plus
   récente ne doit pas priver l'utilisateur de sa liste de conversations. */
async fn read_file() -> SessionOrderFile {
    let Ok(data) = tokio::fs::read_to_string(order_path()).await else {
        return SessionOrderFile::default();
    };
    match serde_json::from_str(&data) {
        Ok(file) => file,
        Err(_) => {
            ::log::warn!("[session_order] fichier illisible, ordre manuel ignoré");
            SessionOrderFile::default()
        }
    }
}

/// Rang de chaque conversation placée à la main, toutes listes confondues.
///
/// Les rangs se répètent d'une liste à l'autre, et c'est sans conséquence :
/// l'affichage filtre par projet avant de lire l'ordre, et un tri stable
/// conserve les positions relatives à l'intérieur de chaque liste.
pub async fn ranks() -> HashMap<String, usize> {
    let file = read_file().await;
    let mut ranks = HashMap::new();
    for ids in file.lists.values() {
        for (rank, id) in ids.iter().enumerate() {
            ranks.insert(id.clone(), rank);
        }
    }
    ranks
}

/// Remplace l'ordre d'une liste entière. `project_id` absent désigne les
/// conversations hors projet.
pub async fn set(project_id: Option<&str>, ids: Vec<String>) -> Result<(), String> {
    write_list(project_id.unwrap_or(ORPHAN_LIST), ids).await
}

/// Remplace l'ordre de la liste des conversations épinglées.
pub async fn set_pinned(ids: Vec<String>) -> Result<(), String> {
    write_list(PINNED_LIST, ids).await
}

/// Retire une conversation de toutes les listes. Une conversation ne doit
/// jamais porter deux rangs — `ranks()` les fusionne — et quitter une liste
/// sans rang la fait remonter en tête de la suivante, par activité.
pub async fn clear_rank(id: &str) -> Result<(), String> {
    super::session_store::validate_session_id(id)?;
    let mut file = read_file().await;
    let mut changed = false;
    for ids in file.lists.values_mut() {
        let before = ids.len();
        ids.retain(|candidate| candidate != id);
        changed |= ids.len() != before;
    }
    if !changed {
        return Ok(());
    }
    write_file(file).await
}

async fn write_list(key: &str, ids: Vec<String>) -> Result<(), String> {
    for id in &ids {
        super::session_store::validate_session_id(id)?;
    }
    let mut file = read_file().await;
    file.lists.insert(key.to_string(), ids);
    write_file(file).await
}

async fn write_file(mut file: SessionOrderFile) -> Result<(), String> {
    file.version = VERSION;
    prune_dead_lists(&mut file).await;
    let data = serde_json::to_string_pretty(&file)
        .map_err(|_| "Enregistrement de l'ordre impossible".to_string())?;
    crate::services::private_store::atomic_write_async(order_path(), data.into_bytes()).await
}

/* Un projet supprimé laisserait sa liste ici pour toujours. Le nettoyage se
   fait à l'écriture, seul moment où le fichier est déjà ouvert. */
async fn prune_dead_lists(file: &mut SessionOrderFile) {
    let Ok(projects) = super::project_store::list().await else {
        return;
    };
    let alive: HashSet<String> = projects.into_iter().map(|p| p.id).collect();
    file.lists
        .retain(|key, _| key == ORPHAN_LIST || key == PINNED_LIST || alive.contains(key));
}

#[cfg(test)]
pub(crate) fn test_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    &LOCK
}

#[path = "session_order_tests.rs"]
#[cfg(test)]
mod tests;
