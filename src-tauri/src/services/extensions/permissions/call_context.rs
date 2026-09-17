use super::host_identity::HostIdentity;
use super::types::ExtensionApiLevel;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct ExtensionCallContext {
    identity: HostIdentity,
    api_level: ExtensionApiLevel,
    generation: u64,
    correlation_id: Uuid,
    revoked: CancellationToken,
    capabilities: Vec<String>,
    core_scope: Option<Arc<super::core_scope::AuthorizedCoreScope>>,
    core_scope_error: Option<&'static str>,
}

impl ExtensionCallContext {
    pub(super) fn from_bound_channel(
        identity: HostIdentity,
        api_level: ExtensionApiLevel,
        generation: u64,
        revoked: CancellationToken,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            identity,
            api_level,
            generation,
            correlation_id: Uuid::new_v4(),
            revoked,
            capabilities,
            core_scope: None,
            core_scope_error: None,
        }
    }

    pub(super) fn identity(&self) -> &HostIdentity {
        &self.identity
    }

    pub(super) fn api_level(&self) -> &ExtensionApiLevel {
        &self.api_level
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    pub(super) fn revoked(&self) -> &CancellationToken {
        &self.revoked
    }

    pub(super) fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|value| value == capability)
    }

    pub(super) fn core_scope(&self) -> Option<&Arc<super::core_scope::AuthorizedCoreScope>> {
        self.core_scope.as_ref()
    }

    pub(super) fn core_scope_error(&self) -> Option<&'static str> {
        self.core_scope_error
    }

    pub(super) fn with_core_scope(
        mut self,
        scope: Arc<super::core_scope::AuthorizedCoreScope>,
    ) -> Self {
        self.core_scope = Some(scope);
        self
    }

    pub(super) fn with_core_scope_error(mut self, reason: &'static str) -> Self {
        self.core_scope_error = Some(reason);
        self
    }

    #[cfg(test)]
    pub(super) fn for_test(identity: HostIdentity, api_level: ExtensionApiLevel) -> Self {
        Self::from_bound_channel(identity, api_level, 1, CancellationToken::new(), Vec::new())
    }

    #[cfg(test)]
    pub(super) fn for_test_with_capabilities(
        identity: HostIdentity,
        api_level: ExtensionApiLevel,
        capabilities: Vec<String>,
    ) -> Self {
        Self::from_bound_channel(
            identity,
            api_level,
            1,
            CancellationToken::new(),
            capabilities,
        )
    }
}
