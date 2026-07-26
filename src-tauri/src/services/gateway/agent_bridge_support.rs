use sha2::{Digest, Sha256};
use tauri::Emitter;

use super::agent_bridge::BridgeError;
use super::channels::{ChannelAdapter, InboundMessage, OutboundMessage};
use super::security::{
    audit::{self, AuditAction},
    ids, validation,
};
use super::session_map;
use crate::models::{ChannelAccountConfig, GatewayConfig};
use crate::services::agent_local::session_store;

pub(crate) fn resolve_provider_model(
    account: &ChannelAccountConfig,
    config: &GatewayConfig,
) -> (String, String) {
    if !account.provider.is_empty() && !account.model.is_empty() {
        return (account.provider.clone(), account.model.clone());
    }
    (
        config.default_provider.clone(),
        config.default_model.clone(),
    )
}

pub(crate) fn validate_inbound(msg: &InboundMessage) -> Result<(), BridgeError> {
    ids::validate_channel_id(&msg.channel_key.channel_id).map_err(BridgeError::Blocked)?;
    ids::validate_account_id(&msg.channel_key.account_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.user_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.message_id).map_err(BridgeError::Blocked)?;
    ids::validate_external_id(&msg.chat_id).map_err(BridgeError::Blocked)?;
    if let Some(thread_id) = &msg.thread_id {
        ids::validate_external_id(thread_id).map_err(BridgeError::Blocked)?;
    }
    let vr = validation::validate_message(&msg.content);
    if !vr.valid {
        return Err(BridgeError::Blocked(vr.reason.unwrap_or_default()));
    }
    Ok(())
}

pub(crate) fn block(msg: &InboundMessage, reason: &str) -> BridgeError {
    match audit_msg(msg, AuditAction::Blocked, Some(reason), None) {
        Ok(()) => BridgeError::Blocked(reason.to_string()),
        Err(error) => error,
    }
}

pub(crate) fn audit_msg(
    msg: &InboundMessage,
    action: AuditAction,
    decision: Option<&str>,
    error: Option<&str>,
) -> Result<(), BridgeError> {
    audit::log_gateway_action(
        &msg.channel_key.channel_id,
        &msg.channel_key.account_id,
        &msg.user_id,
        action,
        decision,
        error,
    )
    .map_err(|_| BridgeError::AuditError)
}

pub(crate) fn emit_session_updated(app: &tauri::AppHandle, session_id: &str) {
    let payload = serde_json::json!({ "session_id": session_id });
    let _ = app.emit("agent-session-updated", &payload);
    let _ = app.emit("wakeup-completed", &payload);
}

pub(crate) async fn sync_session_model(session_id: &str, provider: &str, model: &str) {
    if let Ok(session) = session_store::get(session_id).await {
        if session.provider != provider || session.model != model {
            let _ = session_store::update_model(session_id, model, provider, None, None).await;
        }
    }
}

pub(crate) async fn send_final_reply(
    msg: &InboundMessage,
    adapter: &dyn ChannelAdapter,
    final_messages: &[crate::services::agent_local::types_ollama::ChatMessage],
) -> Result<(), BridgeError> {
    let Some(reply) = super::stream_capture::extract_final_reply(final_messages) else {
        return Ok(());
    };
    let max_utf16 = adapter.capabilities().max_message_chars;
    for chunk in super::stream_capture::prepare_for_channel(&reply, max_utf16) {
        let result = adapter
            .send(OutboundMessage {
                chat_id: msg.chat_id.clone(),
                thread_id: msg.thread_id.clone(),
                content: chunk,
                reply_to: Some(msg.message_id.clone()),
            })
            .await;
        if let Err(error) = result {
            audit_msg(msg, AuditAction::MessageSent, None, Some(&error.message))?;
            return Err(BridgeError::SendError(error.message));
        }
    }
    audit_msg(msg, AuditAction::MessageSent, None, None)?;
    Ok(())
}

pub(crate) fn find_account_config(
    config: &GatewayConfig,
    msg: &InboundMessage,
) -> Option<ChannelAccountConfig> {
    let accounts = match msg.channel_key.channel_id.as_str() {
        "telegram" => &config.channels.telegram,
        "slack" => &config.channels.slack,
        "discord" => &config.channels.discord,
        _ => return None,
    };
    accounts
        .iter()
        .find(|a| a.enabled && a.account_id == msg.channel_key.account_id)
        .cloned()
}

pub(crate) fn build_external_key(msg: &InboundMessage) -> String {
    let mut hasher = Sha256::new();
    for value in [
        msg.channel_key.channel_id.as_str(),
        msg.channel_key.account_id.as_str(),
        msg.user_id.as_str(),
        msg.chat_id.as_str(),
    ] {
        hash_component(&mut hasher, value.as_bytes());
    }
    match &msg.thread_id {
        Some(value) => {
            hasher.update([1]);
            hash_component(&mut hasher, value.as_bytes());
        }
        None => hasher.update([0]),
    }
    format!("gateway:v2:{:x}", hasher.finalize())
}

fn hash_component(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

pub(crate) async fn find_or_create_session(
    msg: &InboundMessage,
    channel_key: &str,
    provider: &str,
    model: &str,
    max_sessions: usize,
    app: &tauri::AppHandle,
) -> Result<String, BridgeError> {
    if let Some(id) = session_map::find(channel_key).await {
        if session_store::get(&id).await.is_ok() {
            return Ok(id);
        }
    }
    let label = match msg.channel_key.channel_id.as_str() {
        "telegram" => "TG",
        "slack" => "SL",
        "discord" => "DC",
        _ => "GW",
    };
    let name = format!("[{label}] {}", msg.user_id);
    let session = session_store::create_gateway(&name, model, provider, channel_key.to_string())
        .await
        .map_err(BridgeError::SessionError)?;
    session_map::insert_bounded(channel_key, &session.id, max_sessions)
        .await
        .map_err(BridgeError::SessionError)?;
    emit_session_updated(app, &session.id);
    Ok(session.id)
}

#[cfg(test)]
#[path = "agent_bridge_support_tests.rs"]
mod tests;
