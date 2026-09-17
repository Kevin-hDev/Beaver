use tokio::sync::watch;

static NEXT_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SubagentTerminalState {
    pub generation: u64,
    pub sequence: u64,
    pub report_persistence_failed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentTerminalKind {
    ReportPersisted,
    ReportPersistenceFailed,
}

#[derive(Clone)]
pub struct SubagentTerminalNotifier {
    sender: watch::Sender<SubagentTerminalState>,
}

impl SubagentTerminalNotifier {
    pub fn notify(&self, kind: SubagentTerminalKind) {
        self.sender.send_modify(|state| {
            state.sequence = state.sequence.saturating_add(1);
            state.report_persistence_failed |=
                kind == SubagentTerminalKind::ReportPersistenceFailed;
        });
    }

    pub fn subscribe(&self) -> watch::Receiver<SubagentTerminalState> {
        self.sender.subscribe()
    }

    pub fn state(&self) -> SubagentTerminalState {
        *self.sender.borrow()
    }

    pub fn reset(&self) {
        self.sender.send_modify(|state| {
            state.sequence = 0;
            state.report_persistence_failed = false;
        });
    }
}

pub fn notifier() -> SubagentTerminalNotifier {
    let generation = NEXT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let (sender, _) = watch::channel(SubagentTerminalState {
        generation,
        ..Default::default()
    });
    SubagentTerminalNotifier { sender }
}
