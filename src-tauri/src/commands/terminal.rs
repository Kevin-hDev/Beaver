use crate::services::terminal::{terminal_error, PtyChannelEvent, PtyManager};
use tauri::ipc::Channel;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtySpawnResult {
    pub id: u32,
    pub token: String,
}

/* Ouvrir un terminal ferme d'abord ceux dont le shell est déjà mort, et chaque
   fermeture attend l'arrêt de son arbre de processus : plusieurs centaines de
   millisecondes en tout.

   Sans `async`, Tauri exécute une commande sur le fil qui dessine la fenêtre.
   L'application y restait figée le temps de ce ménage, et le panneau du
   terminal ne commençait à se déplier qu'une fois celui-ci terminé — une à deux
   secondes après le clic. Le travail est le même, il se fait ailleurs. */
#[tauri::command]
pub async fn pty_spawn(
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    on_output: Channel<PtyChannelEvent>,
    state: State<'_, PtyManager>,
) -> Result<PtySpawnResult, String> {
    let manager = state.inner().clone();
    let (id, token) = tauri::async_runtime::spawn_blocking(move || {
        manager.spawn(on_output, cwd.as_deref(), cols, rows)
    })
    .await
    .map_err(|_| terminal_error())??;
    Ok(PtySpawnResult { id, token })
}

#[tauri::command]
pub fn pty_write(
    id: u32,
    token: String,
    data: String,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    state.write(id, &token, data.as_bytes())
}

#[tauri::command]
pub fn pty_resize(
    id: u32,
    token: String,
    cols: u16,
    rows: u16,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    state.resize(id, &token, cols, rows)
}

/* Fermer coûte la même attente qu'ouvrir, et pour la même raison : l'arrêt de
   l'arbre de processus du shell. Sur le fil qui dessine, la fenêtre se figeait
   à chaque onglet refermé. */
#[tauri::command]
pub async fn pty_kill(
    id: u32,
    token: String,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let manager = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.kill(id, &token))
        .await
        .map_err(|_| terminal_error())?
}
