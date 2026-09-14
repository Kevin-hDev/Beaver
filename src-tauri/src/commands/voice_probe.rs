use crate::services::voice_probe::{VoiceProbeRuntime, VoiceProbeSnapshot};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn voice_probe_available() -> bool {
    true
}

#[tauri::command]
pub fn voice_probe_start(
    app: AppHandle,
    runtime: State<'_, VoiceProbeRuntime>,
) -> Result<VoiceProbeSnapshot, String> {
    runtime
        .start(app)
        .map_err(|error| error.public_code().to_string())
}

#[tauri::command]
pub fn voice_probe_stop(
    runtime: State<'_, VoiceProbeRuntime>,
) -> Result<VoiceProbeSnapshot, String> {
    runtime
        .stop()
        .map_err(|error| error.public_code().to_string())
}
