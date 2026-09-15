use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use super::{
    capture::{
        activity::CaptureStopReason, session::CaptureSession, window_events::WindowEventState,
    },
    download::{VoiceCatalogEntry, VoiceEngine, VoiceModelRole},
    errors::VoiceError,
    model::recognizer::{EffectiveLanguage, ExecutionProfile},
    runtime::VoiceRuntime,
    transcription::transcribe,
    types::{VoiceLanguage, VoiceModel, VoicePhase, VoiceSettings},
};

const CHANGED_EVENT: &str = "voice-state-changed";
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn spawn(
    app: AppHandle,
    runtime: VoiceRuntime,
    operation_id: String,
    settings: VoiceSettings,
    language: Option<VoiceLanguage>,
) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = run(&app, &runtime, &operation_id, &settings, language.as_ref()) {
            runtime
                .coordinator_for_pipeline()
                .fail(&operation_id, error);
            emit(&app, &runtime);
        }
    });
}

fn run(
    app: &AppHandle,
    runtime: &VoiceRuntime,
    operation_id: &str,
    settings: &VoiceSettings,
    language: Option<&VoiceLanguage>,
) -> Result<(), VoiceError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| VoiceError::configuration_unavailable())?;
    let catalog = super::download::load_catalog(&resource_dir)?;
    let asr = find_model(&catalog.entries, settings.model)?;
    let vad = catalog
        .entries
        .iter()
        .find(|item| item.role == VoiceModelRole::Vad)
        .ok_or_else(VoiceError::configuration_unavailable)?;
    let threads = std::thread::available_parallelism().map_or(1, |count| count.get().min(8)) as i32;
    let mut model = runtime.models().acquire_for_capture(
        &crate::services::paths::data_dir(),
        asr,
        vad,
        ExecutionProfile::cpu(threads)?,
        settings.unload_delay,
    )?;
    if runtime.snapshot().phase == VoicePhase::Stopping {
        runtime
            .coordinator_for_pipeline()
            .stop_without_result(operation_id)?;
        emit(app, runtime);
        return Ok(());
    }
    // Silero is recurrent: a new capture must not inherit the previous capture's state.
    model.vad().reset();
    let windows = app.state::<WindowEventState>();
    let mut capture = CaptureSession::open(&settings.input_device, false, true, 1)?;
    runtime
        .coordinator_for_pipeline()
        .begin_listening(operation_id)?;
    emit(app, runtime);
    let started = Instant::now();
    loop {
        std::thread::sleep(POLL_INTERVAL);
        model.poll_loading()?;
        let phase = runtime.snapshot().phase;
        if matches!(
            phase,
            VoicePhase::Transcribing | VoicePhase::Recovering | VoicePhase::Stopping
        ) {
            break;
        }
        let poll = capture.poll(
            model.vad(),
            &windows,
            settings.silence_timeout,
            settings.max_duration,
        )?;
        let capture_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        runtime.coordinator_for_pipeline().record_capture(
            operation_id,
            capture_ms,
            poll.speech_ms,
            poll.level.lost_samples > 0,
            // La crête, pas la moyenne : la moyenne d'une voix normale reste
            // sous 0,1 et dessinerait un signal plat à l'écran.
            poll.level.peak,
        )?;
        emit(app, runtime);
        if let Some(reason) = poll.stop_reason {
            match reason {
                CaptureStopReason::NoSpeech => {
                    runtime
                        .coordinator_for_pipeline()
                        .stop_without_result(operation_id)?;
                    emit(app, runtime);
                    return Ok(());
                }
                CaptureStopReason::Silence
                | CaptureStopReason::DurationLimit
                | CaptureStopReason::Validated => {
                    runtime.coordinator_for_pipeline().validate(operation_id)?;
                }
                CaptureStopReason::Disconnected | CaptureStopReason::HiddenOrLocked => {
                    runtime
                        .coordinator_for_pipeline()
                        .cancel_insertion(operation_id)?;
                }
            }
            emit(app, runtime);
            break;
        }
    }
    let phase = runtime.snapshot().phase;
    if phase == VoicePhase::Stopping {
        // Le micro ferme tout de suite, mais l'opération reste visible et
        // réservée jusqu'au retour du chargement natif déjà engagé.
        drop(capture);
        drop(model.into_ready());
        runtime
            .coordinator_for_pipeline()
            .stop_without_result(operation_id)?;
        emit(app, runtime);
        return Ok(());
    }
    let (mut audio, speech_ms) = capture.finish(model.vad())?;
    let capture_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    runtime.coordinator_for_pipeline().record_capture(
        operation_id,
        capture_ms,
        speech_ms,
        audio.lost_samples() > 0,
        0.0,
    )?;
    let mut lease = model.into_ready()?;
    let effective = effective_language(asr.engine, language.unwrap_or(&settings.language));
    let compute_started = Instant::now();
    let result = transcribe(&mut lease, &mut audio, &effective)?;
    let compute_ms = u64::try_from(compute_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let now_ms = runtime.coordinator_for_pipeline().now_ms();
    runtime.coordinator_for_pipeline().complete_with_metrics(
        operation_id,
        result.text,
        now_ms,
        compute_ms,
    )?;
    emit(app, runtime);
    Ok(())
}

fn find_model(
    entries: &[VoiceCatalogEntry],
    model: VoiceModel,
) -> Result<&VoiceCatalogEntry, VoiceError> {
    let id = match model {
        VoiceModel::ParakeetTdtV3 => "parakeet-tdt-v3-int8",
        VoiceModel::CohereTranscribe => "cohere-transcribe-int8",
        VoiceModel::Qwen3Asr06b => "qwen3-asr-06b-int8",
    };
    entries
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(VoiceError::configuration_unavailable)
}

fn effective_language(engine: VoiceEngine, language: &VoiceLanguage) -> EffectiveLanguage {
    match (engine, language) {
        (VoiceEngine::CohereTranscribe, VoiceLanguage::Language(code)) => {
            EffectiveLanguage::Language(code.clone())
        }
        _ => EffectiveLanguage::Automatic,
    }
}

fn emit(app: &AppHandle, runtime: &VoiceRuntime) {
    let _ = app.emit(CHANGED_EVENT, runtime.snapshot());
}
