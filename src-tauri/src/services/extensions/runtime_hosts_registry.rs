use std::sync::Arc;

use super::super::host_identity::HostIdentity;
use super::super::host_process::HostProcess;
use super::super::types::{ExtensionApiLevel, MAX_HOST_PROCESSES};
use super::{BoundHostChannel, HostGeneration, HostReservation, RuntimeHosts};

impl RuntimeHosts {
    pub(in crate::services::extensions) fn reserve(
        &mut self,
        identity: HostIdentity,
    ) -> Result<HostReservation, String> {
        if self.len() >= MAX_HOST_PROCESSES || self.contains(&identity) {
            return Err(super::super::error_codes::LIMIT_REACHED.to_string());
        }
        let prefix = match identity {
            HostIdentity::Official => "official-",
            HostIdentity::ThirdParty(_) => "third-party-",
        };
        let temporary_directory = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(&self.temporary_root)
            .map_err(|_| super::super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let generation = Arc::new(HostGeneration::new(self.next_generation));
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| super::super::error_codes::HOST_UNAVAILABLE.to_string())?;
        Ok(HostReservation {
            identity,
            generation,
            revoked: tokio_util::sync::CancellationToken::new(),
            temporary_directory,
            exit_sender: self.exit_sender.clone(),
        })
    }

    pub(in crate::services::extensions) fn revoke_reservation(
        &self,
        reservation: &HostReservation,
    ) {
        reservation.revoked.cancel();
    }

    pub(in crate::services::extensions) fn bind(
        &mut self,
        reservation: HostReservation,
        api_level: ExtensionApiLevel,
        process: Arc<HostProcess>,
    ) -> Result<(), HostReservation> {
        if self.len() >= MAX_HOST_PROCESSES || self.contains(&reservation.identity) {
            return Err(reservation);
        }
        let channel = BoundHostChannel {
            identity: reservation.identity.clone(),
            api_level,
            generation: Arc::clone(&reservation.generation),
            process,
            revoked: reservation.revoked,
            _temporary_directory: reservation.temporary_directory,
        };
        match &reservation.identity {
            HostIdentity::Official => self.official = Some(channel),
            HostIdentity::ThirdParty(id) => {
                self.third_party.insert(id.clone(), channel);
            }
        }
        Ok(())
    }

    pub(in crate::services::extensions) fn retain_failed(
        &mut self,
        reservation: HostReservation,
        api_level: ExtensionApiLevel,
        process: Arc<HostProcess>,
    ) {
        reservation.revoked.cancel();
        reservation.generation.begin_stop(false);
        self.failed.push(BoundHostChannel {
            identity: reservation.identity,
            api_level,
            generation: reservation.generation,
            process,
            revoked: reservation.revoked,
            _temporary_directory: reservation.temporary_directory,
        });
        debug_assert!(self.len() <= MAX_HOST_PROCESSES);
    }

    pub(in crate::services::extensions) fn channel(
        &self,
        identity: &HostIdentity,
    ) -> Option<&BoundHostChannel> {
        match identity {
            HostIdentity::Official => self.official.as_ref(),
            HostIdentity::ThirdParty(id) => self.third_party.get(id),
        }
    }

    pub(super) fn owned_channel(&self, identity: &HostIdentity) -> Option<&BoundHostChannel> {
        self.channel(identity).or_else(|| {
            self.failed
                .iter()
                .find(|channel| channel.identity == *identity)
        })
    }

    pub(in crate::services::extensions) fn len(&self) -> usize {
        usize::from(self.official.is_some()) + self.third_party.len() + self.failed.len()
    }

    fn contains(&self, identity: &HostIdentity) -> bool {
        self.channel(identity).is_some()
            || self
                .failed
                .iter()
                .any(|channel| channel.identity == *identity)
    }
}
