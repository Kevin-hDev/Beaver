use crate::services::terminal::{
    caller,
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
pub async fn load_terminal_tabs(
    webview: tauri::WebviewWindow,
) -> Result<TerminalTabsDocument, String> {
    caller::authorize(webview.label())?;
    tab_store::load().await
}

#[tauri::command]
pub async fn save_terminal_tabs(
    webview: tauri::WebviewWindow,
    document: TerminalTabsDocument,
) -> Result<(), String> {
    caller::authorize(webview.label())?;
    tab_store::save(document).await
}

#[tauri::command]
pub async fn pty_spawn(
    webview: tauri::WebviewWindow,
    group_key: String,
    cols: u16,
    rows: u16,
    on_output: Channel<PtyChannelEvent>,
    state: State<'_, PtyManager>,
) -> Result<PtySpawnResult, String> {
    let owner = caller::authorize(webview.label())?;
    let cwd = crate::services::terminal::cwd_resolver::resolve(&group_key).await?;
    let manager = state.inner().clone();
    #[cfg(target_os = "linux")]
    let (id, token) = manager
        .spawn_linux(owner, on_output, cwd, cols, rows)
        .await?;
    #[cfg(not(target_os = "linux"))]
    let (id, token) = super::terminal_blocking::run(move || {
        manager.spawn(&owner, on_output, Some(cwd.as_path()), cols, rows)
    })
    .await?;
    Ok(PtySpawnResult { id, token })
}

#[tauri::command]
pub async fn pty_write(
    webview: tauri::WebviewWindow,
    id: u32,
    token: String,
    data: String,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let owner = caller::authorize(webview.label())?;
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.write(&owner, id, &token, data.as_bytes())).await
}

#[tauri::command]
pub async fn pty_resize(
    webview: tauri::WebviewWindow,
    id: u32,
    token: String,
    cols: u16,
    rows: u16,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let owner = caller::authorize(webview.label())?;
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.resize(&owner, id, &token, cols, rows)).await
}

#[tauri::command]
pub async fn pty_ack_output(
    webview: tauri::WebviewWindow,
    id: u32,
    token: String,
    sequence: u32,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let owner = caller::authorize(webview.label())?;
    state.acknowledge(&owner, id, &token, sequence)
}

#[tauri::command]
pub async fn pty_kill(
    webview: tauri::WebviewWindow,
    id: u32,
    token: String,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    let owner = caller::authorize(webview.label())?;
    let manager = state.inner().clone();
    super::terminal_blocking::run(move || manager.kill(&owner, id, &token)).await
}
