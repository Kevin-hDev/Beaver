use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

const MAX_RUNTIME_LOOPS: usize = 4;
const MAX_RUNTIME_TASKS: usize = 8;

type RuntimeLoopWork = ServiceWorkSupervisor<MAX_RUNTIME_LOOPS>;
type RuntimeTaskWork = ServiceWorkSupervisor<MAX_RUNTIME_TASKS>;

#[derive(Clone)]
pub struct RuntimeBackgroundServices {
    loops: RuntimeLoopWork,
    tasks: RuntimeTaskWork,
}

impl RuntimeBackgroundServices {
    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            loops: RuntimeLoopWork::new(app.clone()),
            tasks: RuntimeTaskWork::new(app),
        }
    }

    pub fn spawn_loop<Factory, Task>(&self, work: Factory) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.loops.spawn(work)
    }

    pub fn spawn_task<Factory, Task>(&self, work: Factory) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.tasks.spawn(work)
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let (loops, tasks) = tokio::join!(
            self.loops.stop_and_wait(deadline),
            self.tasks.stop_and_wait(deadline),
        );
        loops && tasks
    }
}
