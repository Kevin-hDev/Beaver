mod api;
pub(crate) mod api_capabilities;
mod api_images;
mod api_tools;
pub(crate) mod common;
mod compress;
mod context_usage_seed;
mod conversation;
mod gemma4_thinking_guard;
mod ollama;
mod ollama_setup;
mod ollama_thinking;
mod params;
mod prompt_settings;
mod reasoning_diagnostics;
mod session_events;
pub(crate) mod tool_policy;
mod workspace_prompt;

#[cfg(test)]
pub(crate) use conversation::convert as convert_provider_message_for_test;
pub(crate) use conversation::StreamConversation;
pub(crate) use params::{StreamCapabilityHints, StreamPermissionMode, StreamTaskParams};

use crate::services::agent_local::agent_loop_finish::CompletedStreamTurn;
use crate::services::mascot::MascotSessionOutcome;
use std::future::Future;
use std::pin::Pin;

pub(crate) use common::merge_personality;

#[derive(Debug, PartialEq, Eq)]
enum ChatEngine {
    Ollama,
    NativeApi,
}

fn chat_engine(provider: &str) -> ChatEngine {
    if provider == "ollama" {
        ChatEngine::Ollama
    } else {
        ChatEngine::NativeApi
    }
}

pub(crate) type SpawnedStreamTask =
    Pin<Box<dyn Future<Output = Result<CompletedStreamTurn, String>> + Send + 'static>>;

pub(crate) fn run_stream_task(params: StreamTaskParams) -> SpawnedStreamTask {
    let mascot_session = params.on_event.start_mascot_session();
    let inner = Box::pin(run_stream_task_inner(params));
    Box::pin(async move {
        let result = inner.await;
        if let Some(session) = mascot_session {
            session.finish(mascot_outcome(&result));
        }
        result
    })
}

async fn run_stream_task_inner(
    mut params: StreamTaskParams,
) -> Result<CompletedStreamTurn, String> {
    if let Some(permission_emitter) = params.permission_emitter.take() {
        params.on_event = params.on_event.with_permission_emitter(permission_emitter);
    }
    validate_canonical_target(&params)?;
    let conversation = params
        .conversation
        .take()
        .ok_or_else(|| "conversation_admission_failed".to_string())?;
    let (messages, mut journal) = conversation
        .into_messages_and_journal(params.session_id.clone(), params.request_id.clone())?;
    if compress::is_compress_command(&messages) {
        let working_dir = common::resolve_working_dir(&params.working_dir)?;
        common::update_working_dir(&params.session_id, &working_dir).await?;
        compress::handle_compress_command(
            &params.on_event,
            &params.session_id,
            &params.request_id,
            &messages,
            &params.model,
            &params.provider,
            &working_dir,
            params.cancel.clone(),
        )
        .await?;
        return Ok(CompletedStreamTurn::compression(messages));
    }

    let mode = common::resolve_permission_mode(&params.permission_mode).await;
    let response_language = common::response_language();
    session_events::emit_started(&params.session_id, &mode.mode);

    if chat_engine(&params.provider) == ChatEngine::Ollama {
        ollama::run(params, messages, mode, response_language, &mut journal).await
    } else {
        api::run(params, messages, mode, response_language, &mut journal).await
    }
}

fn validate_canonical_target(params: &StreamTaskParams) -> Result<(), String> {
    let Some(target) = params.continuation_target.as_ref() else {
        return Ok(());
    };
    let Some(profile) = params.reasoning_profile.as_ref() else {
        return Err("conversation_admission_failed".to_string());
    };
    validate_target_profile(
        &params.provider,
        &params.model,
        target,
        profile,
        params.think,
        params.reasoning_mode.as_deref(),
    )
}

pub(crate) fn validate_target_profile(
    provider: &str,
    model: &str,
    target: &crate::services::reasoning_continuity::contract::ContinuationTarget,
    profile: &crate::services::reasoning_profile::EffectiveReasoningProfile,
    think: bool,
    reasoning_mode: Option<&str>,
) -> Result<(), String> {
    // Autorité finale : `think`, `reasoning_mode`, profil et cible sont issus de
    // la même résolution ; tout couple incohérent est refusé avant le transport.
    let route =
        crate::services::reasoning_continuity::contract::RouteId::from_provider_id(provider);
    let mode = serde_json::to_value(target.reasoning_mode())
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned));
    let payload_matches =
        if target.route_id() == crate::services::reasoning_continuity::contract::RouteId::Ollama {
            profile.ollama_payload.is_some()
        } else {
            profile.ollama_payload.is_none()
        };
    let profile_matches = profile.mode == target.reasoning_mode()
        && profile.active == think
        && profile.mode_name.as_deref() == reasoning_mode
        && payload_matches;
    if target.validate().is_err()
        || route != Some(target.route_id())
        || target.model_id() != model
        || mode.as_deref() != Some(reasoning_mode.unwrap_or("off"))
        || !profile_matches
    {
        return Err("conversation_admission_failed".to_string());
    }
    Ok(())
}

fn mascot_outcome(result: &Result<CompletedStreamTurn, String>) -> MascotSessionOutcome {
    match result {
        Ok(_) => MascotSessionOutcome::Success,
        Err(message) if message == "Annulé" => MascotSessionOutcome::Cancelled,
        Err(_) => MascotSessionOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grok_and_kimi_oauth_use_the_native_agent_loop() {
        assert_eq!(chat_engine("xai-oauth"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("moonshot-oauth"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("xai"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("moonshot"), ChatEngine::NativeApi);
    }

    #[test]
    fn mascot_outcome_covers_every_terminal_path() {
        assert_eq!(
            mascot_outcome(&Ok(CompletedStreamTurn::compression(Vec::new()))),
            MascotSessionOutcome::Success
        );
        assert_eq!(
            mascot_outcome(&Err("Annulé".into())),
            MascotSessionOutcome::Cancelled
        );
        assert_eq!(
            mascot_outcome(&Err("indisponible".into())),
            MascotSessionOutcome::Failed
        );
    }

    #[test]
    fn every_stream_consumer_receives_a_boxed_agent_loop() {
        type StreamRun = fn(StreamTaskParams) -> SpawnedStreamTask;

        // La coercition échoue à la compilation si la boucle redevient non boxed.
        let _run: StreamRun = run_stream_task;
    }
}
