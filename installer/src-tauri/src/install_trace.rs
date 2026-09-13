use crate::error::InstallerError;
use crate::trace::{ErrorCode, InstallerTrace, OperationId, Outcome, Phase, TraceEntry};
use std::sync::Mutex;
use std::time::Instant;

pub struct TraceSession {
    trace: Mutex<Option<InstallerTrace>>,
}

impl TraceSession {
    pub fn new() -> Self {
        Self {
            trace: Mutex::new(None),
        }
    }

    pub fn start(&self, run: &std::path::Path) -> Result<Instant, InstallerError> {
        let mut trace = self
            .trace
            .lock()
            .map_err(|_| InstallerError::CleanupFailed)?;
        if trace.is_none() {
            *trace = Some(InstallerTrace::create(run)?);
        }
        let started = Instant::now();
        record(&mut trace, started, Phase::Started, Outcome::Pending, None);
        Ok(started)
    }

    pub fn finish(&self, started: Instant, outcome: Outcome, error: Option<InstallerError>) {
        if let Ok(mut trace) = self.trace.lock() {
            record(
                &mut trace,
                started,
                Phase::Finished,
                outcome,
                error.map(error_code),
            );
        }
    }

    pub fn take(&self) -> Option<InstallerTrace> {
        self.trace.lock().ok().and_then(|mut trace| trace.take())
    }
}

fn record(
    trace: &mut Option<InstallerTrace>,
    started: Instant,
    phase: Phase,
    outcome: Outcome,
    error_code: Option<ErrorCode>,
) {
    if let Some(trace) = trace {
        let _ = trace.record(TraceEntry {
            operation_id: OperationId::Install,
            phase,
            outcome,
            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            error_code,
        });
    }
}

fn error_code(error: InstallerError) -> ErrorCode {
    match error {
        InstallerError::InvalidLaunch => ErrorCode::InvalidLaunch,
        InstallerError::DownloadFailed => ErrorCode::DownloadFailed,
        InstallerError::IntegrityFailed => ErrorCode::IntegrityFailed,
        InstallerError::CleanupFailed => ErrorCode::CleanupFailed,
        InstallerError::InstallFailed => ErrorCode::InstallFailed,
    }
}
