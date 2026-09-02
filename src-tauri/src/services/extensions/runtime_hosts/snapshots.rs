use super::{BoundHostChannel, RuntimeHosts};
use crate::services::extensions::host_identity::HostIdentity;
use crate::services::extensions::host_process::HostProcess;
use crate::services::extensions::types::ExtensionApiLevel;
use std::sync::Arc;

impl RuntimeHosts {
    pub(in crate::services::extensions) fn authorize_loads(
        &self,
        identity: &HostIdentity,
        process: &Arc<HostProcess>,
        specifications: &[super::super::protocol::HostExtensionSpec],
    ) -> Result<(), String> {
        for specification in specifications {
            self.authorize_load(identity, process, &specification.id)?;
        }
        Ok(())
    }

    pub(in crate::services::extensions) fn authorize_load(
        &self,
        identity: &HostIdentity,
        process: &Arc<HostProcess>,
        extension_id: &str,
    ) -> Result<(), String> {
        let channel = self
            .channel(identity)
            .filter(|channel| {
                !channel.revoked.is_cancelled() && Arc::ptr_eq(&channel.process, process)
            })
            .ok_or_else(|| super::super::error_codes::HOST_UNAVAILABLE.to_string())?;
        if channel.identity != *identity
            || matches!(identity, HostIdentity::ThirdParty(id) if id != extension_id)
        {
            return Err(super::super::error_codes::REQUEST_INVALID.to_string());
        }
        Ok(())
    }

    pub(in crate::services::extensions) fn snapshots(
        &self,
    ) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        self.official
            .iter()
            .chain(self.third_party.values())
            .chain(self.failed.iter())
            .map(|channel| {
                (
                    channel.identity.clone(),
                    channel.generation.number,
                    Arc::clone(&channel.process),
                )
            })
            .collect()
    }

    pub(in crate::services::extensions) fn bound_snapshots(
        &self,
    ) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        self.official
            .iter()
            .chain(self.third_party.values())
            .map(|channel| {
                (
                    channel.identity.clone(),
                    channel.generation.number,
                    Arc::clone(&channel.process),
                )
            })
            .collect()
    }

    pub(in crate::services::extensions) fn snapshot(
        &self,
        identity: &HostIdentity,
    ) -> Option<(ExtensionApiLevel, u64, Arc<HostProcess>)> {
        self.owned_channel(identity).map(channel_snapshot)
    }

    pub(in crate::services::extensions) fn usable_snapshot(
        &self,
        identity: &HostIdentity,
    ) -> Option<(ExtensionApiLevel, u64, Arc<HostProcess>)> {
        self.channel(identity)
            .filter(|channel| !channel.revoked.is_cancelled() && !channel.generation.is_stopping())
            .map(channel_snapshot)
    }

    pub(in crate::services::extensions) fn usable_snapshots(
        &self,
    ) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        self.official
            .iter()
            .chain(self.third_party.values())
            .filter(|channel| !channel.revoked.is_cancelled() && !channel.generation.is_stopping())
            .map(|channel| {
                (
                    channel.identity.clone(),
                    channel.generation.number,
                    Arc::clone(&channel.process),
                )
            })
            .collect()
    }

    pub(in crate::services::extensions) fn stop_is_confirmed(
        &self,
        identity: &HostIdentity,
        generation: u64,
    ) -> bool {
        self.channel(identity)
            .is_none_or(|channel| channel.generation.number != generation)
            && !self.failed.iter().any(|channel| {
                channel.identity == *identity && channel.generation.number == generation
            })
    }

    pub(in crate::services::extensions) fn call_context(
        &self,
        identity: &HostIdentity,
        generation: u64,
    ) -> Option<crate::services::extensions::call_context::ExtensionCallContext> {
        self.channel(identity)
            .filter(|channel| {
                channel.generation.number == generation && !channel.revoked.is_cancelled()
            })
            .map(BoundHostChannel::call_context)
    }
}

fn channel_snapshot(channel: &BoundHostChannel) -> (ExtensionApiLevel, u64, Arc<HostProcess>) {
    (
        channel.api_level.clone(),
        channel.generation.number,
        Arc::clone(&channel.process),
    )
}
