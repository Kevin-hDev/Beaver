mod api;
pub(crate) mod api_capabilities;
mod api_images;
mod api_tools;
pub(crate) mod common;
mod compress;
mod context_lifecycle;
mod context_usage_seed;
mod conversation;
mod fixture_prompt;
mod gemma4_thinking_guard;
mod ollama;
mod ollama_setup;
mod ollama_thinking;
mod params;
mod prompt_settings;
mod reasoning_diagnostics;
#[path = "agent_chat_task_recovery.rs"]
mod recovery;
mod session_events;
pub(crate) mod tool_policy;
mod workspace_prompt;

#[cfg(test)]
pub(crate) use conversation::convert as convert_provider_message_for_test;
pub(crate) use conversation::StreamConversation;
pub(crate) use params::{StreamCapabilityHints, StreamPermissionMode, StreamTaskParams};

use crate::services::agent_local::agent_loop_finish::CompletedStreamTurn;

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

pub(crate) use recovery::SpawnedStreamTask;

pub(crate) fn run_stream_task(params: StreamTaskParams) -> SpawnedStreamTask {
    let recovery_session_id = params.session_id.clone();
    let recovery_request_id = params.request_id.clone();
    let recovery_cancel = params.cancel.clone();
    let mascot_session = params.on_event.start_mascot_session();
    #[cfg(debug_assertions)]
    let fixture_limits = params.fixture_run.as_ref().map(|run| run.limits());
    #[cfg(debug_assertions)]
    let fixture_cancel = params.cancel.clone();
    let inner = Box::pin(run_stream_task_inner(params));
    Box::pin(async move {
        let guarded = recovery::guard(async move {
            #[cfg(debug_assertions)]
            {
                match fixture_limits {
                    Some(limits) => {
                        crate::services::reasoning_fixture_budget::run_scoped(
                            limits,
                            fixture_cancel,
                            inner,
                        )
                        .await
                    }
                    None => inner.await,
                }
            }
            #[cfg(not(debug_assertions))]
            {
                inner.await
            }
        })
        .await;
        let result = recovery::finish(
            guarded,
            &recovery_session_id,
            &recovery_request_id,
            &recovery_cancel,
        )
        .await;
        if let Some(session) = mascot_session {
            session.finish(recovery::mascot_outcome(&result));
        }
        result
    })
}

async fn run_stream_task_inner(
    mut params: StreamTaskParams,
) -> Result<CompletedStreamTurn, String> {
    crate::services::agent_local::tool_bash_security::initialize()
        .await
        .map_err(|_| "stream_error".to_string())?;
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
    if let Some(current) = journal.as_mut() {
        let (log, owner) =
            crate::services::agent_local::stream_recovery_log::StreamRecoveryLog::create(
                current.recovery_header(),
                params.cancel.clone(),
            )
            .await
            .map_err(|_| "stream_error".to_string())?;
        params.on_event = params.on_event.with_recovery_log(log.clone());
        current.attach_recovery(log, owner);
    }
    context_lifecycle::activate(journal.as_ref()).await?;
    let outcome = context_lifecycle::run(params, messages, &mut journal).await;
    context_lifecycle::finish(journal.as_ref(), &outcome).await?;
    outcome
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

#[cfg(test)]
#[path = "agent_chat_task_tests.rs"]
mod tests;
