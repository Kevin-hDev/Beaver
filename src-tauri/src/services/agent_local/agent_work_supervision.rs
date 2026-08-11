use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::Instant;

pub const MAX_ACTIVE_AGENT_STREAMS: usize = 32;
pub const MAX_ACTIVE_SUBAGENTS: usize = 8;
pub const MAX_ACTIVE_SHELLS: usize = 64;
const SUBAGENT_DISPATCHERS: usize = 1;

pub type AgentStreamWork = ServiceWorkSupervisor<MAX_ACTIVE_AGENT_STREAMS>;
pub type SubagentWork = ServiceWorkSupervisor<MAX_ACTIVE_SUBAGENTS>;
pub type ShellWork = ServiceWorkSupervisor<MAX_ACTIVE_SHELLS>;
pub type SubagentDispatcherWork = ServiceWorkSupervisor<SUBAGENT_DISPATCHERS>;

pub struct AgentWorkServices {
    streams: AgentStreamWork,
    subagents: SubagentWork,
    shells: ShellWork,
    subagent_dispatcher: SubagentDispatcherWork,
}

impl AgentWorkServices {
    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            streams: AgentStreamWork::new(app.clone()),
            subagents: SubagentWork::new(app.clone()),
            shells: ShellWork::new(app.clone()),
            subagent_dispatcher: SubagentDispatcherWork::new(app),
        }
    }

    pub fn streams(&self) -> AgentStreamWork {
        self.streams.clone()
    }

    pub fn subagents(&self) -> SubagentWork {
        self.subagents.clone()
    }

    pub fn shells(&self) -> ShellWork {
        self.shells.clone()
    }

    pub fn subagent_dispatcher(&self) -> SubagentDispatcherWork {
        self.subagent_dispatcher.clone()
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let (streams, subagents, shells, dispatcher) = tokio::join!(
            self.streams.stop_and_wait(deadline),
            self.subagents.stop_and_wait(deadline),
            self.shells.stop_and_wait(deadline),
            self.subagent_dispatcher.stop_and_wait(deadline),
        );
        streams && subagents && shells && dispatcher
    }
}
