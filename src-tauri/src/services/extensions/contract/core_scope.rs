use super::host_identity::HostIdentity;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_tool_profile::SubagentToolProfile;
use crate::services::llm::request_purpose::RequestPurpose;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use subtle::ConstantTimeEq;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use zeroize::Zeroizing;

const SECRET_BYTES: usize = 32;

#[derive(Clone)]
pub(crate) struct AgentCoreScope {
    pub session_id: String,
    pub request_id: String,
    pub working_directory: PathBuf,
    pub permission_mode: String,
    pub profile: Option<SubagentToolProfile>,
    pub purpose: RequestPurpose,
    pub cancel: CancellationToken,
    pub on_event: AgentEventEmitter,
    pub plan_active: bool,
}

pub(super) struct AuthorizedCoreScope {
    pub(super) identity: HostIdentity,
    pub(super) generation: u64,
    pub(super) agent: AgentCoreScope,
    pub(super) tool_name: String,
    pub(super) tool_effect: super::types::ExtensionEffect,
    pub(super) deadline: Instant,
    secret: Zeroizing<[u8; SECRET_BYTES]>,
}

#[derive(Clone, Default)]
pub(super) struct CoreScopeRegistry {
    entries: Arc<Mutex<BTreeMap<Uuid, Arc<AuthorizedCoreScope>>>>,
}

pub(super) struct CoreScopeLease {
    registry: CoreScopeRegistry,
    id: Uuid,
    scope: Arc<AuthorizedCoreScope>,
}

pub(super) struct CoreScopeEnvelope<'a> {
    id: Uuid,
    secret: &'a [u8; SECRET_BYTES],
    remaining_ms: u64,
}

impl CoreScopeRegistry {
    pub(super) fn admit(
        &self,
        identity: HostIdentity,
        generation: u64,
        agent: AgentCoreScope,
        tool_name: String,
        tool_effect: super::types::ExtensionEffect,
        deadline: Instant,
    ) -> Result<CoreScopeLease, &'static str> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if entries.len() >= super::types::MAX_ACTIVE_CONTEXTS
            || entries
                .values()
                .filter(|scope| scope.identity == identity)
                .count()
                >= super::types::MAX_CONTEXTS_PER_HOST_IDENTITY
        {
            return Err("core_saturated");
        }
        let id = Uuid::new_v4();
        let mut secret = Zeroizing::new([0_u8; SECRET_BYTES]);
        crate::services::secure_random::try_fill(secret.as_mut())
            .map_err(|_| "core_request_failed")?;
        let scope = Arc::new(AuthorizedCoreScope {
            identity,
            generation,
            agent,
            tool_name,
            tool_effect,
            deadline,
            secret,
        });
        entries.insert(id, Arc::clone(&scope));
        Ok(CoreScopeLease {
            registry: self.clone(),
            id,
            scope,
        })
    }

    pub(super) fn resolve(
        &self,
        id: &str,
        encoded_secret: &str,
        identity: &HostIdentity,
        generation: u64,
    ) -> Result<Arc<AuthorizedCoreScope>, &'static str> {
        let id = Uuid::parse_str(id).map_err(|_| "core_context_invalid")?;
        let mut supplied = Zeroizing::new([0_u8; SECRET_BYTES]);
        hex::decode_to_slice(encoded_secret, supplied.as_mut())
            .map_err(|_| "core_context_invalid")?;
        let scope = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
            .cloned()
            .ok_or("core_context_expired")?;
        if !bool::from(scope.secret.ct_eq(supplied.as_ref()))
            || scope.identity != *identity
            || scope.generation != generation
        {
            return Err("core_context_invalid");
        }
        if scope.agent.cancel.is_cancelled() {
            return Err("core_context_revoked");
        }
        if Instant::now() >= scope.deadline {
            return Err("core_context_expired");
        }
        Ok(scope)
    }
}

impl CoreScopeLease {
    pub(super) fn envelope(&self) -> CoreScopeEnvelope<'_> {
        let remaining_ms = self
            .scope
            .deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        CoreScopeEnvelope {
            id: self.id,
            secret: &self.scope.secret,
            remaining_ms,
        }
    }
}

impl Drop for CoreScopeLease {
    fn drop(&mut self) {
        self.registry
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&self.id);
    }
}

impl Serialize for CoreScopeEnvelope<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut encoded = Zeroizing::new([0_u8; SECRET_BYTES * 2]);
        hex::encode_to_slice(self.secret, encoded.as_mut()).map_err(serde::ser::Error::custom)?;
        let secret = std::str::from_utf8(encoded.as_ref()).map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct("CoreScopeEnvelope", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("secret", secret)?;
        state.serialize_field("remainingMs", &self.remaining_ms)?;
        state.end()
    }
}
