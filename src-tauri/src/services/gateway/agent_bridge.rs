use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::models::GatewayConfig;
use crate::services::gateway::agent_bridge_support::{
    audit_msg, block, build_external_key, find_account_config, find_or_create_session,
    resolve_provider_model, sync_session_model, validate_inbound,
};
use crate::services::gateway::channels::{ChannelAdapter, InboundMessage};
use crate::services::gateway::conversation_locks::ConversationLocks;
use crate::services::gateway::security::{
    allowlist::Allowlist, audit::AuditAction, rate_state::GatewayRateLimiters,
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
        cancel: CancellationToken,
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
            audit_msg(
                &msg,
                AuditAction::RateLimited,
                Some("rate_limited"),
                Some("rate_limited"),
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
        if !crate::services::llm::stream_dispatch::is_available(
            &provider,
            crate::services::llm::stream_dispatch::InvocationKind::Interactive,
            crate::services::llm::request_purpose::RequestPurpose::ExternalChannel,
        ) {
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
        super::agent_bridge_run::run(
            app,
            &msg,
            adapter.as_ref(),
            cancel,
            session_id,
            provider,
            model,
        )
        .await
    }
}

#[cfg(test)]
#[path = "agent_bridge_tests.rs"]
mod tests;
