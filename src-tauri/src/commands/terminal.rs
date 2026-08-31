use crate::services::terminal::{
    tab_store::{self, TerminalTabsDocument},
    PtyChannelEvent, PtyManager,
};
use tauri::ipc::Channel;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtySpawnResult {
    pub id: u32,
    pub token: String,
}

#[tauri::command]
pub async fn load_terminal_tabs() -> Result<TerminalTabsDocument, String> {
    tab_store::load().await
}

#[tauri::command]
pub async fn save_terminal_tabs(document: TerminalTabsDocument) -> Result<(), String> {
    tab_store::save(document).await
}

#[tauri::command]
pub async fn pty_spawn(
    group_key: String,
    cols: u16,
    rows: u16,
    on_output: Channel<PtyChannelEvent>,
    state: State<'_, PtyManager>,
) -> Result<PtySpawnResult, String> {
    let cwd = crate::services::terminal::cwd_resolver::resolve(&group_key).await?;
    let manager = state.inner().clone();
    // Provisoire : le lanceur durable Linux du plan I/O remplacera cette
    // frontière avant l'ajout de PR_SET_PDEATHSIG.
    let (id, token) = tauri::async_runtime::spawn_blocking(move || {
        manager.spawn(on_output, Some(cwd.as_path()), cols, rows)
    })
    .await
    .map_err(|_| "terminal-error".to_string())??;
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

#[tauri::command]
pub fn pty_kill(id: u32, token: String, state: State<'_, PtyManager>) -> Result<(), String> {
    state.kill(id, &token)
}
