use crate::services::work_registry::ServiceWorkAdmissionError;
use chrono::{DateTime, Local};

pub(super) struct DueAdmissionOutcome {
    pub(super) keep_running: bool,
    pub(super) decision_persisted: bool,
}

pub(super) async fn handle_due_admission<Recorder, RecordFuture>(
    result: Result<(), ServiceWorkAdmissionError>,
    wakeup_id: String,
    scheduled_for: DateTime<Local>,
    record: Recorder,
) -> DueAdmissionOutcome
where
    Recorder: FnOnce(String, DateTime<Local>, ServiceWorkAdmissionError) -> RecordFuture,
    RecordFuture: std::future::Future<Output = Result<(), String>>,
{
    let Err(error) = result else {
        return DueAdmissionOutcome {
            keep_running: true,
            decision_persisted: true,
        };
    };
    let decision_persisted = warn_if_log_failed(record(wakeup_id, scheduled_for, error).await);
    let keep_running = match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => false,
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            ::log::warn!("[scheduler] capacité des réveils atteinte");
            true
        }
    };
    DueAdmissionOutcome {
        keep_running,
        decision_persisted,
    }
}

pub(super) async fn persist_once_missed_decision<Recorder, RecordFuture, Claimer>(
    record: Recorder,
    claim: Claimer,
) -> Result<(), String>
where
    Recorder: FnOnce() -> RecordFuture,
    RecordFuture: std::future::Future<Output = Result<(), String>>,
    Claimer: FnOnce() -> Result<(), String>,
{
    // The journal is the durable decision authority; config cleanup follows it.
    record().await?;
    claim()
}

pub(super) fn warn_if_log_failed(result: Result<(), String>) -> bool {
    if result.is_ok() {
        true
    } else {
        ::log::warn!("[scheduler] journal indisponible");
        false
    }
}
