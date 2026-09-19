use tauri::{AppHandle, Emitter, Manager};

use crate::services::{
    model_downloads::{ModelDownloadKind, ModelDownloadManager},
    voice::{
        contracts::{VoiceAction, VoiceCatalogItem, VoiceDevice, VoiceSnapshot},
        errors::VoiceError,
        runtime::VoiceRuntime,
        types::{VoiceSettings, VoiceSettingsPatch},
    },
};

#[tauri::command]
pub fn voice_get_snapshot(voice: tauri::State<'_, VoiceRuntime>) -> VoiceSnapshot {
    voice.snapshot()
}

#[tauri::command]
pub fn voice_get_settings() -> Result<VoiceSettings, VoiceError> {
    crate::services::voice::settings::read_voice_settings()
}

#[tauri::command]
pub fn voice_get_catalog(
    app: AppHandle,
    voice: tauri::State<'_, VoiceRuntime>,
) -> Result<Vec<VoiceCatalogItem>, VoiceError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| VoiceError::configuration_unavailable())?;
    let data_dir = crate::services::paths::data_dir();
    let threads = std::thread::available_parallelism().map_or(1, |count| count.get().min(8)) as i32;
    let profile = crate::services::voice::model::recognizer::ExecutionProfile::cpu(threads)?;
    let entries = crate::services::voice::download::load_catalog(&resource_dir)?.entries;
    let vad_installed = entries
        .iter()
        .find(|entry| entry.role == crate::services::voice::download::VoiceModelRole::Vad)
        .ok_or_else(VoiceError::configuration_unavailable)
        .and_then(|entry| {
            crate::services::voice::download::installed_receipt(entry, &data_dir)
                .map(|receipt| receipt.is_some())
                .map_err(|_| VoiceError::configuration_unavailable())
        })?;
    entries
        .into_iter()
        .map(|entry| {
            use crate::services::voice::{download::VoiceEngine, types::VoiceModel};
            let model = match entry.engine {
                VoiceEngine::NemoTransducer => Some(VoiceModel::ParakeetTdtV3),
                VoiceEngine::CohereTranscribe => Some(VoiceModel::CohereTranscribe),
                VoiceEngine::Qwen3Asr => Some(VoiceModel::Qwen3Asr06b),
                VoiceEngine::SileroVad => None,
            };
            let installed = crate::services::voice::download::installed_receipt(&entry, &data_dir)
                .map_err(|_| VoiceError::configuration_unavailable())?
                .is_some()
                && (model.is_none() || vad_installed);
            let speed_multiplier = match voice.models().current_benchmark(
                &data_dir,
                &entry.id,
                &entry.revision,
                &profile,
            ) {
                Ok(benchmark) => benchmark.map(|value| value.multiplier()),
                Err(code) => {
                    ::log::warn!(
                        "[voice] model={} step=benchmark-read-failed code={code}",
                        entry.id
                    );
                    None
                }
            };
            Ok(VoiceCatalogItem {
                id: entry.id,
                model,
                installed,
                download_bytes: entry.archive.bytes,
                installed_bytes: entry.installed_bytes,
                languages: entry.languages,
                dialects: entry.dialects,
                language_mode: entry.language_mode,
                speed_multiplier,
            })
        })
        .collect()
}

#[tauri::command]
pub async fn voice_dispatch(
    app: AppHandle,
    action: VoiceAction,
    voice: tauri::State<'_, VoiceRuntime>,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<VoiceSnapshot, VoiceError> {
    let snapshot = match action {
        VoiceAction::Install { model_id } => {
            super::start_model_download(
                app.clone(),
                ModelDownloadKind::Voice,
                model_id,
                Some(false),
                downloads,
            )
            .await
            .map_err(|_| VoiceError::configuration_unavailable())?;
            voice.coordinator_for_command().clear_error()
        }
        VoiceAction::Resume { transfer_id } => {
            super::resume_model_download(app.clone(), transfer_id, downloads)
                .await
                .map_err(|_| VoiceError::configuration_unavailable())?;
            voice.snapshot()
        }
        VoiceAction::CancelDownload { transfer_id } => {
            super::cancel_model_download(app.clone(), transfer_id, downloads)
                .await
                .map_err(|_| VoiceError::configuration_unavailable())?;
            voice.snapshot()
        }
        VoiceAction::Uninstall { model_id } => {
            voice.remove_model(&model_id)?;
            voice.snapshot()
        }
        action => {
            let start_settings = if matches!(action, VoiceAction::Start { .. }) {
                let settings = crate::services::voice::settings::read_voice_settings()?;
                if !settings.enabled || !settings.explanation_accepted {
                    return Err(VoiceError::invalid_settings());
                }
                Some(settings)
            } else {
                None
            };
            let start_request = match (&action, &start_settings) {
                (VoiceAction::Start { language, .. }, Some(settings)) => {
                    Some((settings.clone(), language.clone()))
                }
                _ => None,
            };
            let snapshot = voice.dispatch_with_settings(
                action,
                main_window_is_foreground(&app),
                start_settings,
            )?;
            if let (Some(operation), Some((settings, language))) =
                (snapshot.operation.as_ref(), start_request)
            {
                crate::services::voice::pipeline::spawn(
                    app.clone(),
                    voice.inner().clone(),
                    operation.id.clone(),
                    settings,
                    language,
                );
            }
            snapshot
        }
    };
    let _ = app.emit(
        crate::services::voice::contracts::VOICE_CHANGED_EVENT,
        &snapshot,
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn voice_update_settings(
    app: AppHandle,
    patch: VoiceSettingsPatch,
    voice: tauri::State<'_, VoiceRuntime>,
) -> Result<VoiceSettings, VoiceError> {
    let model_guard = patch
        .model
        .is_some()
        .then(|| voice.coordinator_for_command());
    if model_guard
        .as_ref()
        .is_some_and(|coordinator| coordinator.snapshot().operation.is_some())
    {
        return Err(VoiceError::busy());
    }
    let disabled = patch.enabled == Some(false);
    let settings = crate::services::voice::settings::update_voice_settings(patch)?;
    drop(model_guard);
    if disabled {
        let snapshot = voice.coordinator_for_command().disable();
        let _ = app.emit(
            crate::services::voice::contracts::VOICE_CHANGED_EVENT,
            snapshot,
        );
    }
    Ok(settings)
}

#[tauri::command]
pub fn voice_list_devices() -> Result<Vec<VoiceDevice>, VoiceError> {
    crate::services::voice::capture::device::list()
}

fn main_window_is_foreground(app: &AppHandle) -> bool {
    !crate::services::voice::capture::window_events::session_is_locked()
        && app
            .get_webview_window("main")
            .and_then(|window| window.is_focused().ok())
            .unwrap_or(false)
}
