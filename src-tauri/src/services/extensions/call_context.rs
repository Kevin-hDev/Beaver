use super::host_identity::HostIdentity;
use super::types::ExtensionApiLevel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct ExtensionCallContext {
    identity: HostIdentity,
    api_level: ExtensionApiLevel,
    generation: u64,
    correlation_id: Uuid,
    revoked: CancellationToken,
}

impl ExtensionCallContext {
    pub(super) fn from_bound_channel(
        identity: HostIdentity,
        api_level: ExtensionApiLevel,
        generation: u64,
        revoked: CancellationToken,
    ) -> Self {
        Self {
            identity,
            api_level,
            generation,
            correlation_id: Uuid::new_v4(),
            revoked,
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

    #[cfg(test)]
    pub(super) fn for_test(identity: HostIdentity, api_level: ExtensionApiLevel) -> Self {
        Self::from_bound_channel(identity, api_level, 1, CancellationToken::new())
    }
}
