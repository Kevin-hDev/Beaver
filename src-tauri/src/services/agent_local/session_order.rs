/* Ordre choisi à la main dans une liste de conversations — celles d'un projet,
   ou celles qui n'appartiennent à aucun.

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
    for id in &ids {
        super::session_store::validate_session_id(id)?;
    }
    let key = project_id.unwrap_or(ORPHAN_LIST).to_string();
    let mut file = read_file().await;
    file.version = VERSION;
    file.lists.insert(key, ids);
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
        .retain(|key, _| key == ORPHAN_LIST || alive.contains(key));
}

#[path = "session_order_tests.rs"]
#[cfg(test)]
mod tests;
