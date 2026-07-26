use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::GatewayConfig;
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::{self, AgentEventEmitter};
use crate::services::gateway::agent_bridge_support::{
    audit_msg, block, build_external_key, emit_session_updated, find_account_config,
    find_or_create_session, resolve_provider_model, send_final_reply, sync_session_model,
    validate_inbound,
};
use crate::services::gateway::channels::{ChannelAdapter, InboundMessage};
use crate::services::gateway::conversation_locks::ConversationLocks;
use crate::services::gateway::message_convert;
use crate::services::gateway::security::{
    allowlist::Allowlist,
    audit::{self, AuditAction},
    rate_state::GatewayRateLimiters,
};

#[derive(Debug)]
pub enum BridgeError {
    Blocked(String),
    AuditError,
    SessionError(String),
    AgentError(String),
    SendError(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(r) => write!(f, "blocked: {r}"),
            Self::AuditError => f.write_str("audit: unavailable"),
            Self::SessionError(e) => write!(f, "session: {e}"),
            Self::AgentError(e) => write!(f, "agent: {e}"),
            Self::SendError(e) => write!(f, "send: {e}"),
        }
    }
}

pub struct GatewayAgentBridge {
    limits: Arc<Mutex<GatewayRateLimiters>>,
    conversations: ConversationLocks,
}

impl GatewayAgentBridge {
    pub fn new(limits: Arc<Mutex<GatewayRateLimiters>>, max_conversations: usize) -> Self {
        Self {
            limits,
            conversations: ConversationLocks::new(max_conversations),
        }
    }

    fn read_config() -> GatewayConfig {
        crate::services::config::read_config()
            .map(|c| c.gateway)
            .unwrap_or_default()
    }

    pub async fn process(
        &self,
        msg: InboundMessage,
        adapter: Arc<dyn ChannelAdapter>,
        app: tauri::AppHandle,
    ) -> Result<(), BridgeError> {
        let config = Self::read_config();
        validate_inbound(&msg)?;
        audit_msg(&msg, AuditAction::MessageReceived, None, None)?;

        let account_cfg = find_account_config(&config, &msg)
            .ok_or_else(|| block(&msg, "account not configured"))?;
        let al = Allowlist::from_list(&account_cfg.allowlist, false);
        if !al.contains(&msg.user_id) {
            return Err(block(&msg, "user not in allowlist"));
        }
        let decision = self.limits.lock().await.consume(&msg);
        if !decision.allowed {
            let retry = format!(
                "retry_after_ms={};remaining={}",
                decision.retry_after_ms, decision.remaining
            );
            audit_msg(
                &msg,
                AuditAction::RateLimited,
                Some("rate_limited"),
                Some(&retry),
            )?;
            return Err(BridgeError::Blocked("rate limited".into()));
        }

        let channel_key = build_external_key(&msg);
        let _conversation_guard = self
            .conversations
            .acquire(&channel_key)
            .await
            .map_err(|reason| block(&msg, &reason))?;
        let (provider, model) = resolve_provider_model(&account_cfg, &config);
        if crate::services::llm::route::is_interactive_only(&provider) {
            return Err(block(&msg, "provider restricted to interactive chat"));
        }
        let session_id = find_or_create_session(
            &msg,
            &channel_key,
            &provider,
            &model,
            config.max_sessions as usize,
            &app,
        )
        .await?;

        sync_session_model(&session_id, &provider, &model).await;

        let session = session_store::get(&session_id)
            .await
            .map_err(BridgeError::SessionError)?;
        let mut messages = message_convert::build_chat_messages(&session);
        let history_len = messages.len();
        messages.push(message_convert::new_user_message(&msg.content));
        session_store::add_messages(
            &session_id,
            vec![message_convert::new_user_agent_message(&msg.content)],
            0,
        )
        .await
        .map_err(BridgeError::SessionError)?;
        emit_session_updated(&app, &session_id);

        let generation = stream_events::next_generation();
        let emitter =
            AgentEventEmitter::with_generation(app.clone(), session_id.clone(), generation);
        let request_id = crate::services::agent_local::stream_diagnostics::start_request(
            &session_id,
            generation,
        )
        .await;
        let final_messages = match run_stream_task(StreamTaskParams {
            on_event: emitter,
            session_id: session_id.clone(),
            request_id: request_id.clone(),
            model,
            messages,
            tools: vec![],
            think: false,
            provider,
            working_dir: None,
            capability_hints: StreamCapabilityHints::default(),
            reasoning_mode: None,
            permission_mode_override: Some("auto".to_string()),
            permission_emitter: None,
            parent_message_inbox: None,
            subagent_profile: None,
            plan_mode: Some(false),
            cancel: CancellationToken::new(),
        })
        .await
        {
            Ok(messages) => messages,
            Err(e) => {
                crate::services::agent_local::stream_diagnostics::record_failure(
                    &session_id,
                    Some(&request_id),
                    &e,
                    false,
                )
                .await;
                let safe = audit::sanitize_error(&e);
                audit_msg(&msg, AuditAction::AgentError, None, Some(&safe))?;
                return Err(BridgeError::AgentError(safe));
            }
        };

        let new_assistant_messages: Vec<_> = {
            let mut non_system = final_messages.iter().filter(|m| m.role != "system");
            for _ in 0..history_len {
                non_system.next();
            }
            non_system.next();
            non_system
                .filter_map(message_convert::chat_to_agent_message)
                .collect()
        };

        session_store::add_messages(&session_id, new_assistant_messages, 0)
            .await
            .map_err(BridgeError::SessionError)?;
        emit_session_updated(&app, &session_id);

        send_final_reply(&msg, adapter.as_ref(), &final_messages).await?;
        Ok(())
    }
}
