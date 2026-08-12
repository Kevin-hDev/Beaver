use super::due::{due_wakeups_at, is_late, is_once, missed_occurrences};
use super::next_fire::next_fire_at;
use super::work_supervision::SchedulerWakeupWork;
use super::{fire, log, state};
use crate::services::config::read_config;
use crate::services::work_registry::{ServiceWorkAdmissionError, ServiceWorkCancellation};
use chrono::{DateTime, Duration as ChronoDuration, Local};
use tauri::AppHandle;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

const MAX_SLEEP_MIN: i64 = 60;

pub(super) async fn run_loop(
    app: AppHandle,
    mut reload_rx: watch::Receiver<u64>,
    lifetime: ServiceWorkCancellation,
    wakeup_work: SchedulerWakeupWork,
) {
    loop {
        if lifetime.is_cancelled() {
            return;
        }
        let config = match read_config() {
            Ok(config) => config,
            Err(_) => {
                ::log::warn!("[scheduler] configuration indisponible");
                tokio::select! {
                    _ = lifetime.cancelled() => return,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
                }
                continue;
            }
        };

        let now = Local::now();
        if config.heartbeat.global_paused {
            checkpoint(now).await;
        } else {
            reconcile_missed(&config.scheduled_wakeups, now).await;
        }
        let cap = now + ChronoDuration::minutes(MAX_SLEEP_MIN);
        let next = next_scheduled_at(
            &config.scheduled_wakeups,
            config.heartbeat.global_paused,
            now,
        );
        let target = next.map(|time| time.min(cap)).unwrap_or(cap);
        let sleep = (target - now)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(60));

        tokio::select! {
            _ = lifetime.cancelled() => return,
            _ = tokio::time::sleep(sleep) => {
                if let Some(target) = next {
                    handle_due(
                        app.clone(),
                        &config.scheduled_wakeups,
                        now,
                        target,
                        &lifetime,
                        &wakeup_work,
                    ).await;
                }
            }
            _ = reload_rx.changed() => {}
        }
    }
}

fn next_scheduled_at(
    wakeups: &[crate::models::ScheduledWakeup],
    globally_paused: bool,
    now: DateTime<Local>,
) -> Option<DateTime<Local>> {
    if globally_paused {
        return None;
    }
    wakeups
        .iter()
        .filter(|wakeup| wakeup.active && !wakeup.paused_by_global)
        .filter_map(|wakeup| next_fire_at(&wakeup.schedule, now))
        .min()
}

async fn reconcile_missed(wakeups: &[crate::models::ScheduledWakeup], now: DateTime<Local>) {
    let Some(last_checked) = state::read_last_checked().await else {
        checkpoint(now).await;
        return;
    };
    let mut decisions_persisted = true;
    for (wakeup, scheduled_for) in missed_occurrences(wakeups, last_checked, now) {
        let result = if is_once(&wakeup) {
            match fire::missed_once_action(fire::claim_once(&wakeup.id)) {
                fire::MissedOnceAction::LogMissed => {
                    log::log_missed(&wakeup.id, scheduled_for).await
                }
                fire::MissedOnceAction::Silent => Ok(()),
                fire::MissedOnceAction::LogClaimError(error) => {
                    ::log::warn!("[scheduler] revendication ponctuelle impossible");
                    log::log_err(&wakeup.id, scheduled_for, &error).await
                }
            }
        } else {
            log::log_missed(&wakeup.id, scheduled_for).await
        };
        decisions_persisted &= warn_if_log_failed(result);
    }
    if decisions_persisted {
        checkpoint(now).await;
    }
}

async fn handle_due(
    app: AppHandle,
    wakeups: &[crate::models::ScheduledWakeup],
    loop_now: DateTime<Local>,
    target: DateTime<Local>,
    lifetime: &ServiceWorkCancellation,
    work: &SchedulerWakeupWork,
) {
    if lifetime.is_cancelled() {
        return;
    }
    let current = Local::now();
    if is_late(target, current) {
        reconcile_missed(wakeups, current).await;
        return;
    }

    for wakeup in due_wakeups_at(wakeups, loop_now, target) {
        if lifetime.is_cancelled() {
            return;
        }
        let wakeup_id = wakeup.id.clone();
        let app_clone = app.clone();
        let result = work.spawn(move |service_cancel| async move {
            let cancel = CancellationToken::new();
            let shutdown_cancel = cancel.clone();
            let task = fire::fire_wakeup(app_clone, wakeup, target, cancel);
            tokio::pin!(task);
            tokio::select! {
                biased;
                _ = service_cancel.cancelled() => {
                    shutdown_cancel.cancel();
                    task.await;
                }
                _ = &mut task => {}
            }
        });
        let keep_running = handle_due_admission(
            result,
            wakeup_id,
            target,
            |wakeup_id, scheduled_for, error| async move {
                log::log_refused(&wakeup_id, scheduled_for, error).await
            },
        )
        .await;
        if !keep_running {
            return;
        }
    }
}

pub(super) async fn handle_due_admission<Recorder, RecordFuture>(
    result: Result<(), ServiceWorkAdmissionError>,
    wakeup_id: String,
    scheduled_for: DateTime<Local>,
    record: Recorder,
) -> bool
where
    Recorder: FnOnce(String, DateTime<Local>, ServiceWorkAdmissionError) -> RecordFuture,
    RecordFuture: std::future::Future<Output = Result<(), String>>,
{
    let Err(error) = result else {
        return true;
    };
    warn_if_log_failed(record(wakeup_id, scheduled_for, error).await);
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => false,
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            ::log::warn!("[scheduler] capacité des réveils atteinte");
            true
        }
    }
}

async fn checkpoint(now: DateTime<Local>) -> bool {
    match state::write_last_checked(now).await {
        Ok(()) => true,
        Err(_) => {
            ::log::warn!("[scheduler] état de contrôle indisponible");
            false
        }
    }
}

fn warn_if_log_failed(result: Result<(), String>) -> bool {
    if result.is_ok() {
        true
    } else {
        ::log::warn!("[scheduler] journal indisponible");
        false
    }
}
