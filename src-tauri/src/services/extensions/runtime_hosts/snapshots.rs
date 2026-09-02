use super::{BoundHostChannel, RuntimeHosts};
use crate::services::extensions::host_identity::HostIdentity;
use crate::services::extensions::host_process::HostProcess;
use crate::services::extensions::types::ExtensionApiLevel;
use std::sync::Arc;

impl RuntimeHosts {
    pub(in crate::services::extensions) fn snapshots(
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
        self.channel(identity).map(channel_snapshot)
    }

    pub(in crate::services::extensions) fn usable_snapshot(
        &self,
        identity: &HostIdentity,
    ) -> Option<(ExtensionApiLevel, u64, Arc<HostProcess>)> {
        self.channel(identity)
            .filter(|channel| !channel.revoked.is_cancelled())
            .map(channel_snapshot)
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
