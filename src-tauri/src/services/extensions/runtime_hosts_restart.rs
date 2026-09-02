use super::{HostStartReason, RuntimeHosts};
use crate::services::extensions::host_identity::HostIdentity;
use crate::services::extensions::types::MAX_HOST_PROCESSES;

impl RuntimeHosts {
    pub(in crate::services::extensions) fn admit_spawn(
        &mut self,
        identity: &HostIdentity,
        reason: HostStartReason,
    ) -> bool {
        reason == HostStartReason::InitialOrManual || self.allow_restart(identity)
    }

    pub(in crate::services::extensions) fn allow_restart(
        &mut self,
        identity: &HostIdentity,
    ) -> bool {
        if self
            .channel(identity)
            .is_some_and(|channel| channel.generation.is_stopping())
        {
            return false;
        }
        if !self.restart_budgets.contains_key(identity)
            && self.restart_budgets.len() >= MAX_HOST_PROCESSES
        {
            return false;
        }
        self.restart_budgets
            .entry(identity.clone())
            .or_default()
            .allow()
    }

    pub(in crate::services::extensions) fn reset_restart_budgets(&self) {
        for budget in self.restart_budgets.values() {
            budget.reset();
        }
    }

    pub(in crate::services::extensions) fn forget_restart_budget(
        &mut self,
        identity: &HostIdentity,
    ) {
        self.restart_budgets.remove(identity);
    }
}
