use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkSupervisor,
};
use std::time::Instant;

// Un connecteur configuré peut porter une opération active. Cette capacité
// dérive donc de l'unique limite du catalogue MCP.
pub(super) const MAX_MCP_OPERATIONS: usize = super::config::MAX_CONNECTORS;
pub(super) const MAX_MCP_PROCESSES: usize = 8;

type McpOperationWork = ServiceWorkSupervisor<MAX_MCP_OPERATIONS>;
type McpProcessWork = ServiceWorkSupervisor<MAX_MCP_PROCESSES>;

#[derive(Clone)]
pub(super) struct McpWorkServices {
    operations: McpOperationWork,
    processes: McpProcessWork,
}

impl McpWorkServices {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            operations: McpOperationWork::new(app.clone()),
            processes: McpProcessWork::new(app),
        }
    }

    pub(super) fn try_admit(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_MCP_OPERATIONS>, ServiceWorkAdmissionError> {
        self.operations.try_admit()
    }

    pub(super) fn try_admit_process(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_MCP_PROCESSES>, ServiceWorkAdmissionError> {
        self.processes.try_admit()
    }

    pub(super) fn begin_closing(&self) {
        self.operations.begin_closing();
        self.processes.begin_closing();
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let (operations, processes) = tokio::join!(
            self.operations.stop_and_wait(deadline),
            self.processes.stop_and_wait(deadline),
        );
        operations && processes
    }
}
