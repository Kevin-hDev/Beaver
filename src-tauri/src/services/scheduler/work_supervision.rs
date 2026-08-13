use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkDiagnostics,
    ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

const SCHEDULER_LOOPS: usize = 1;
pub(super) const SCHEDULED_WAKEUPS_CAPACITY: usize = 64;

type SchedulerLoopWork = ServiceWorkSupervisor<SCHEDULER_LOOPS>;
pub(super) type SchedulerWakeupWork = ServiceWorkSupervisor<SCHEDULED_WAKEUPS_CAPACITY>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerDiagnostics {
    pub loop_work: ServiceWorkDiagnostics,
    pub wakeups: ServiceWorkDiagnostics,
}

pub(super) struct SchedulerWorkServices {
    loop_work: SchedulerLoopWork,
    wakeups: SchedulerWakeupWork,
}

impl SchedulerWorkServices {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            loop_work: SchedulerLoopWork::new(app.clone()),
            wakeups: SchedulerWakeupWork::new(app),
        }
    }

    pub(super) fn start_loop<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.loop_work.spawn(work)
    }

    #[cfg(test)]
    pub(super) fn spawn_wakeup<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.wakeups.spawn(work)
    }

    pub(super) fn wakeups(&self) -> SchedulerWakeupWork {
        self.wakeups.clone()
    }

    pub(super) fn diagnostics(&self) -> SchedulerDiagnostics {
        SchedulerDiagnostics {
            loop_work: self.loop_work.diagnostics(),
            wakeups: self.wakeups.diagnostics(),
        }
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.loop_work.begin_closing();
        self.wakeups.begin_closing();
        let (loop_stopped, wakeups_stopped) = tokio::join!(
            self.loop_work.stop_and_wait(deadline),
            self.wakeups.stop_and_wait(deadline),
        );
        loop_stopped && wakeups_stopped
    }
}
