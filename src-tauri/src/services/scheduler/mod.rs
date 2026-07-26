pub mod due;
pub mod fire;
pub mod log;
pub mod next_fire;
#[cfg(test)]
mod next_fire_tests;
pub mod state;

use crate::services::config::read_config;
use chrono::{DateTime, Duration as ChronoDuration, Local};
use due::{due_wakeups_at, is_late, is_once, missed_occurrences};
use next_fire::next_fire_at;
use tauri::AppHandle;
use tokio::sync::watch;

const MAX_SLEEP_MIN: i64 = 60;

pub struct Scheduler {
    reload_tx: watch::Sender<u64>,
}

impl Scheduler {
    pub fn spawn(app: AppHandle) -> Self {
        let (reload_tx, reload_rx) = watch::channel(0u64);
        tauri::async_runtime::spawn(run_loop(app, reload_rx));
        Scheduler { reload_tx }
    }

    pub fn notify_config_changed(&self) {
        let next = self.reload_tx.borrow().wrapping_add(1);
        let _ = self.reload_tx.send(next);
    }
}

async fn run_loop(app: AppHandle, mut reload_rx: watch::Receiver<u64>) {
    loop {
        let cfg = match read_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[scheduler] read_config error: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        let now = Local::now();
        if cfg.heartbeat.global_paused {
            let _ = state::write_last_checked(now).await;
        } else {
            reconcile_missed(&cfg.scheduled_wakeups, now).await;
        }
        let cap = now + ChronoDuration::minutes(MAX_SLEEP_MIN);

        let next: Option<DateTime<Local>> = if cfg.heartbeat.global_paused {
            None
        } else {
            cfg.scheduled_wakeups
                .iter()
                .filter(|w| w.active && !w.paused_by_global)
                .filter_map(|w| next_fire_at(&w.schedule, now))
                .min()
        };

        let target_dt = next.map(|t| t.min(cap)).unwrap_or(cap);
        let sleep_dur = (target_dt - now)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(60));

        tokio::select! {
            _ = tokio::time::sleep(sleep_dur) => {
                if let Some(target) = next {
                    handle_due(app.clone(), &cfg.scheduled_wakeups, now, target).await;
                }
            }
            _ = reload_rx.changed() => {}
        }
    }
}

async fn reconcile_missed(wakeups: &[crate::models::ScheduledWakeup], now: DateTime<Local>) {
    let Some(last_checked) = state::read_last_checked().await else {
        let _ = state::write_last_checked(now).await;
        return;
    };
    for (wakeup, scheduled_for) in missed_occurrences(wakeups, last_checked, now) {
        log::log_missed(&wakeup.id, scheduled_for).await;
        if is_once(&wakeup) {
            let _ = fire::deactivate_once(&wakeup.id);
        }
    }
    let _ = state::write_last_checked(now).await;
}

async fn handle_due(
    app: AppHandle,
    wakeups: &[crate::models::ScheduledWakeup],
    loop_now: DateTime<Local>,
    target: DateTime<Local>,
) {
    let current = Local::now();
    if is_late(target, current) {
        reconcile_missed(wakeups, current).await;
        return;
    }

    let due = due_wakeups_at(wakeups, loop_now, target);
    for wakeup in due {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            fire::fire_wakeup(app_clone, wakeup, target).await;
        });
    }
}
