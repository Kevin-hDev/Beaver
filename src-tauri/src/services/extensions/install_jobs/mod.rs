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
