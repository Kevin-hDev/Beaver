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
    #[cfg(target_os = "linux")]
    let (id, token) = manager.spawn_linux(on_output, cwd, cols, rows).await?;
    #[cfg(not(target_os = "linux"))]
    let (id, token) = super::terminal_blocking::run(move || {
        manager.spawn(on_output, Some(cwd.as_path()), cols, rows)
    })
    .await?;
    Ok(PtySpawnResult { id, token })
}

#[tauri::command]
pub async fn pty_write(
    id: u32,
    token: String,
    data: String,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.write(id, &token, data.as_bytes())).await
}

#[tauri::command]
pub async fn pty_resize(
    id: u32,
    token: String,
    cols: u16,
    rows: u16,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.resize(id, &token, cols, rows)).await
}

#[tauri::command]
pub async fn pty_ack_output(
    id: u32,
    token: String,
    sequence: u32,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    state.acknowledge(id, &token, sequence)
}

#[tauri::command]
pub async fn pty_kill(id: u32, token: String, state: State<'_, PtyManager>) -> Result<(), String> {
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.kill(id, &token)).await
}
