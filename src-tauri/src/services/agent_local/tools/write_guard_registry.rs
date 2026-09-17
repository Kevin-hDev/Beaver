use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Mutex as AsyncMutex;

/// Registre mémoire des `WriteGuard` indexés par `session_id`.
///
/// Permet de faire persister les chemins lus entre les tours d'une même session
/// (même après un nouveau message utilisateur). Sans ce registre, le WriteGuard
/// était réinitialisé à chaque appel de `run_agent_loop`, ce qui forçait l'IA
/// à relire des fichiers qu'elle venait déjà de lire.
///
/// Le registre est perdu au redémarrage de l'app : c'est acceptable pour un
/// cache de "fichiers déjà lus" qui se reconstruit naturellement.
static REGISTRY: LazyLock<Mutex<HashMap<String, Arc<AsyncMutex<WriteGuard>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Limite le nombre de sessions simultanées en mémoire pour éviter une
/// croissance indéfinie (sessions abandonnées, sous-agents, etc.).
/// Au-delà, les sessions les plus anciennes sont évacuées.
const MAX_SESSIONS: usize = 32;

/// Retourne (ou crée) le `WriteGuard` d'une session.
/// Le caller obtient un `Arc<AsyncMutex<WriteGuard>>` qu'il doit locker une
/// fois au début de la boucle agent et passer `&mut` aux exécuteurs de tools.
pub async fn lock(session_id: &str) -> Arc<AsyncMutex<WriteGuard>> {
    {
        let mut map = REGISTRY.lock().expect("REGISTRY poisoned");
        if let Some(guard) = map.get(session_id) {
            return guard.clone();
        }
        // Éviction si trop de sessions
        if map.len() >= MAX_SESSIONS {
            // Retire une clé arbitraire (la première trouvée) — heuristique
            // simple, pas besoin d'ordre précis pour un cache de session.
            if let Some(key) = map.keys().next().cloned() {
                map.remove(&key);
            }
        }
        let guard = Arc::new(AsyncMutex::new(WriteGuard::new()));
        map.insert(session_id.to_string(), guard.clone());
        guard
    }
}

/// Supprime le `WriteGuard` d'une session (à appeler quand la session est
/// supprimée définitivement).
pub fn remove(session_id: &str) {
    let mut map = REGISTRY.lock().expect("REGISTRY poisoned");
    map.remove(session_id);
}
