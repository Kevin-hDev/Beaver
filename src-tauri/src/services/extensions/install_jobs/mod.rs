//! One queue owns installation state; materialization is injected by the installer.
mod admission;
#[cfg(test)]
mod contract_tests;
mod control;
mod limits;
mod request;
mod store;
#[cfg(test)]
mod store_tests;
mod transitions;
mod types;
mod worker;
pub(crate) use control::{InstallControl, InstallInterruption, InstallProgress};
pub(crate) use store::InstallJobStore;
pub(crate) use types::*;
pub(crate) use worker::{InstallExecutor, InstallFuture, InstallOutcome};

pub(crate) fn global() -> Result<InstallJobStore, String> {
    Ok(super::runtime::global()?.install_jobs.clone())
}
mod owned_work;
#[cfg(test)]
mod owned_work_tests;

mod checkpoint;
pub(crate) use checkpoint::protects_artifacts;
mod cleanup;
mod control_checkpoint;
mod executor;
mod journal;
mod materialize;
mod recovery;

mod compatibility;
mod disk_policy;
mod resume;
mod retry_cleanup;

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod publication_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod worker_tests;
