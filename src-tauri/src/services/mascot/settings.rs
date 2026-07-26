use super::{generic_error, window, MascotRuntime, SETTINGS_EVENT};
use crate::models::{ClgoConfig, MascotPosition, MascotSettings, MascotSettingsPatch};
use crate::services::config;
use tauri::{AppHandle, Emitter, Manager};

pub async fn get() -> Result<MascotSettings, String> {
    Ok(read_config().await?.mascot.normalized())
}

pub async fn patch(app: &AppHandle, patch: MascotSettingsPatch) -> Result<MascotSettings, String> {
    let runtime = app.state::<MascotRuntime>();
    let _mutation = runtime.mutation_gate.lock().await;
    let mut config = read_config().await?;
    let previous = config.mascot.clone().normalized();
    let next = patch.apply(previous.clone());
    config.mascot = next.clone();
    write_config(config).await?;

    if window::apply(app, &runtime, &next).is_err() {
        rollback(app, &runtime, previous).await;
        return Err(generic_error());
    }
    store_current(&runtime, next.clone())?;
    let _ = app.emit(SETTINGS_EVENT, &next);
    Ok(next)
}

pub async fn save_position(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    let position = MascotPosition { x, y }
        .normalized()
        .ok_or_else(generic_error)?;
    let runtime = app.state::<MascotRuntime>();
    let _mutation = runtime.mutation_gate.lock().await;
    let mut config = read_config().await?;
    config.mascot.position = Some(position);
    let saved = config.mascot.clone();
    write_config(config).await?;
    store_current(&runtime, saved)
}

pub async fn sync_from_disk(app: &AppHandle) -> Result<(), String> {
    let runtime = app.state::<MascotRuntime>();
    let _mutation = runtime.mutation_gate.lock().await;
    let settings = read_config().await?.mascot.normalized();
    if current(&runtime)? == settings {
        return Ok(());
    }
    window::apply(app, &runtime, &settings)?;
    store_current(&runtime, settings.clone())?;
    let _ = app.emit(SETTINGS_EVENT, settings);
    Ok(())
}

pub fn store_current(runtime: &MascotRuntime, settings: MascotSettings) -> Result<(), String> {
    runtime
        .current_settings
        .lock()
        .map(|mut current| *current = settings)
        .map_err(|_| generic_error())
}

fn current(runtime: &MascotRuntime) -> Result<MascotSettings, String> {
    runtime
        .current_settings
        .lock()
        .map(|current| current.clone())
        .map_err(|_| generic_error())
}

async fn rollback(app: &AppHandle, runtime: &MascotRuntime, previous: MascotSettings) {
    if let Ok(mut config) = read_config().await {
        config.mascot = previous.clone();
        let _ = write_config(config).await;
    }
    let _ = window::apply(app, runtime, &previous);
    let _ = store_current(runtime, previous);
}

async fn read_config() -> Result<ClgoConfig, String> {
    tokio::task::spawn_blocking(config::read_config)
        .await
        .map_err(|_| generic_error())?
        .map_err(|_| generic_error())
}

async fn write_config(config_value: ClgoConfig) -> Result<(), String> {
    tokio::task::spawn_blocking(move || config::write_config(&config_value))
        .await
        .map_err(|_| generic_error())?
        .map_err(|_| generic_error())
}
