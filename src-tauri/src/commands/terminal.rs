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
pub fn pty_spawn(
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    on_output: Channel<PtyChannelEvent>,
    state: State<'_, PtyManager>,
) -> Result<PtySpawnResult, String> {
    let (id, token) = state.spawn(on_output, cwd.as_deref(), cols, rows)?;
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
