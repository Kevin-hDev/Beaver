use super::{HostExitNotice, RuntimeHosts};
use crate::services::extensions::host_identity::HostIdentity;
use crate::services::extensions::host_process::HostProcess;
use std::sync::Arc;

impl RuntimeHosts {
    #[cfg(test)]
    pub(in crate::services::extensions) fn revoke_all(
        &mut self,
    ) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        let snapshots = self.snapshots();
        for (identity, generation, _) in &snapshots {
            let _ = self.begin_stop(identity, *generation, false);
        }
        snapshots
    }

    pub(in crate::services::extensions) fn begin_stop_all(
        &mut self,
        restarting: bool,
    ) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        let snapshots = self.snapshots();
        for (identity, generation, _) in &snapshots {
            let _ = self.begin_stop(identity, *generation, restarting);
        }
        snapshots
    }

    pub(in crate::services::extensions) fn remove_current(
        &mut self,
        identity: &HostIdentity,
        generation: u64,
    ) -> bool {
        if let Some(position) = self.failed.iter().position(|channel| {
            channel.identity == *identity && channel.generation.number == generation
        }) {
            self.failed.remove(position);
            return true;
        }
        if self
            .channel(identity)
            .is_none_or(|channel| channel.generation.number != generation)
        {
            return false;
        }
        if let Some(channel) = self.channel(identity) {
            channel.revoked.cancel();
        }
        match identity {
            HostIdentity::Official => self.official.take(),
            HostIdentity::ThirdParty(id) => self.third_party.remove(id),
        };
        true
    }

    #[cfg(test)]
    pub(in crate::services::extensions) fn revoke(&mut self, extension_id: &str) -> bool {
        let Some(channel) = self.third_party.get(extension_id) else {
            return false;
        };
        channel.revoked.cancel();
        true
    }

    pub(in crate::services::extensions) fn begin_stop(
        &mut self,
        identity: &HostIdentity,
        generation: u64,
        restarting: bool,
    ) -> bool {
        let Some(channel) = self
            .channel(identity)
            .filter(|channel| channel.generation.number == generation)
            .or_else(|| {
                self.failed.iter().find(|channel| {
                    channel.identity == *identity && channel.generation.number == generation
                })
            })
        else {
            return false;
        };
        channel.generation.begin_stop(restarting);
        channel.revoked.cancel();
        true
    }

    pub(in crate::services::extensions) fn exit_kind(
        &self,
        notice: &HostExitNotice,
    ) -> Option<crate::services::extensions::runtime_host_generation::HostExitKind> {
        self.owned_channel(&notice.identity)
            .filter(|channel| channel.generation.number == notice.generation)
            .map(|_| notice.kind)
    }

    pub(in crate::services::extensions) fn remove_stopped(
        &mut self,
        identity: &HostIdentity,
        generation: u64,
        stopped: bool,
    ) -> bool {
        // Le canal reste autoritatif tant que la disparition de l'arbre n'est pas confirmée.
        // Le moniteur de sortie peut avoir récolté cette même génération entre
        // la confirmation OS et cette étape ; elle est alors déjà arrêtée.
        stopped
            && (self.remove_current(identity, generation)
                || self.stop_is_confirmed(identity, generation))
    }
}
