use crate::contract::{InstallerEvent, InstallerSnapshot};
use crate::install::InstallerService;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub fn installer_snapshot(
    service: State<'_, InstallerService>,
) -> CommandResult<InstallerSnapshot> {
    service.snapshot().map_err(public_error)
}

#[tauri::command]
pub async fn choose_install_directory(
    app: AppHandle,
    service: State<'_, InstallerService>,
) -> CommandResult<Option<String>> {
    let selected = app.dialog().file().blocking_pick_folder();
    let Some(path) = selected else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|_| generic_error())?;
    service
        .set_destination(path)
        .map(Some)
        .map_err(public_error)
}

#[tauri::command]
pub async fn start_install(
    on_event: Channel<InstallerEvent>,
    service: State<'_, InstallerService>,
) -> CommandResult<()> {
    service.install(on_event).await.map_err(public_error)
}

#[tauri::command]
pub fn cancel_install(service: State<'_, InstallerService>) -> InstallerEvent {
    service.cancel()
}

#[tauri::command]
pub fn launch_beaver(service: State<'_, InstallerService>) -> CommandResult<()> {
    service.launch_beaver().map_err(public_error)
}

fn public_error(error: crate::error::InstallerError) -> String {
    error.code().to_string()
}

fn generic_error() -> String {
    crate::error::InstallerError::InstallFailed
        .code()
        .to_string()
}
