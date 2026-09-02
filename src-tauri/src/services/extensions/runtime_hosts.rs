use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::types::{ExtensionApiLevel, MAX_HOST_PROCESSES};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct BoundHostChannel {
    pub(super) identity: HostIdentity,
    pub(super) api_level: ExtensionApiLevel,
    pub(super) generation: u64,
    pub(super) process: Arc<HostProcess>,
    _temporary_directory: tempfile::TempDir,
}

pub(super) struct HostReservation {
    identity: HostIdentity,
    generation: u64,
    temporary_directory: tempfile::TempDir,
}

impl HostReservation {
    pub(super) fn temporary_directory(&self) -> &Path {
        self.temporary_directory.path()
    }
}

pub(super) struct RuntimeHosts {
    pub(super) official: Option<BoundHostChannel>,
    pub(super) third_party: BTreeMap<String, BoundHostChannel>,
    temporary_root: PathBuf,
    next_generation: u64,
}

impl RuntimeHosts {
    pub(super) fn new(temporary_root: PathBuf) -> Result<Self, String> {
        purge_orphaned_directories(&temporary_root)?;
        Ok(Self {
            official: None,
            third_party: BTreeMap::new(),
            temporary_root,
            next_generation: 1,
        })
    }

    pub(super) fn reserve(&mut self, identity: HostIdentity) -> Result<HostReservation, String> {
        if self.len() >= MAX_HOST_PROCESSES || self.contains(&identity) {
            return Err(super::error_codes::LIMIT_REACHED.to_string());
        }
        let prefix = match identity {
            HostIdentity::Official => "official-",
            HostIdentity::ThirdParty(_) => "third-party-",
        };
        let temporary_directory = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(&self.temporary_root)
            .map_err(|_| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let generation = self.next_generation;
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        Ok(HostReservation {
            identity,
            generation,
            temporary_directory,
        })
    }

    pub(super) fn bind(
        &mut self,
        reservation: HostReservation,
        api_level: ExtensionApiLevel,
        process: Arc<HostProcess>,
    ) -> Result<(), String> {
        if self.len() >= MAX_HOST_PROCESSES || self.contains(&reservation.identity) {
            return Err(super::error_codes::LIMIT_REACHED.to_string());
        }
        let channel = BoundHostChannel {
            identity: reservation.identity.clone(),
            api_level,
            generation: reservation.generation,
            process,
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

    pub(super) fn channel(&self, identity: &HostIdentity) -> Option<&BoundHostChannel> {
        match identity {
            HostIdentity::Official => self.official.as_ref(),
            HostIdentity::ThirdParty(id) => self.third_party.get(id),
        }
    }

    pub(super) fn snapshots(&self) -> Vec<(HostIdentity, u64, Arc<HostProcess>)> {
        self.official
            .iter()
            .chain(self.third_party.values())
            .map(|channel| {
                (
                    channel.identity.clone(),
                    channel.generation,
                    Arc::clone(&channel.process),
                )
            })
            .collect()
    }

    pub(super) fn snapshot(
        &self,
        identity: &HostIdentity,
    ) -> Option<(ExtensionApiLevel, u64, Arc<HostProcess>)> {
        self.channel(identity).map(|channel| {
            (
                channel.api_level.clone(),
                channel.generation,
                Arc::clone(&channel.process),
            )
        })
    }

    pub(super) fn remove_current(&mut self, identity: &HostIdentity, generation: u64) -> bool {
        if self
            .channel(identity)
            .is_none_or(|channel| channel.generation != generation)
        {
            return false;
        }
        match identity {
            HostIdentity::Official => self.official.take(),
            HostIdentity::ThirdParty(id) => self.third_party.remove(id),
        };
        true
    }

    pub(super) fn remove_stopped(
        &mut self,
        identity: &HostIdentity,
        generation: u64,
        stopped: bool,
    ) -> bool {
        // Un canal reste autoritatif tant que la mort de son processus n'est pas confirmée.
        stopped && self.remove_current(identity, generation)
    }

    pub(super) fn len(&self) -> usize {
        usize::from(self.official.is_some()) + self.third_party.len()
    }

    fn contains(&self, identity: &HostIdentity) -> bool {
        self.channel(identity).is_some()
    }
}

fn purge_orphaned_directories(root: &Path) -> Result<(), String> {
    if root.exists() {
        std::fs::remove_dir_all(root)
            .map_err(|_| super::error_codes::HOST_UNAVAILABLE.to_string())?;
    }
    std::fs::create_dir_all(root).map_err(|_| super::error_codes::HOST_UNAVAILABLE.to_string())
}
