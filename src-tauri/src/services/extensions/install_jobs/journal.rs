use super::{
    checkpoint::{self, Journal},
    InstallJobStore,
};

impl InstallJobStore {
    pub(super) fn persist(&self, state: &super::store::State) -> Result<(), String> {
        let Some(path) = &self.journal else {
            return Ok(());
        };
        let journal = Journal {
            version: checkpoint::FORMAT,
            revision: state.revision,
            jobs: state.jobs.clone(),
        };
        let bytes = serde_json::to_vec(&journal).map_err(|_| super::limits::UNAVAILABLE)?;
        if bytes.len() as u64 > checkpoint::MAX_JOURNAL_BYTES {
            return Err(super::limits::UNAVAILABLE.into());
        }
        crate::services::private_store::atomic_write(path, &bytes)
    }
}
